# Port notes: Minecraft 26.2

This directory is the 26.2 port produced by the version-port pipeline
(`tools/version_port.py`, see `docs/version-ports.md`), ported backward from
the programme's 26.3 base. The completion pass has landed: the protocol
rewire is done, all six generated tables are 26.2's, horizon-prepare runs
end to end, and the workspace compiles clean. What remains is itemised
below with its current state. Nothing in this file is a claim that the
port is finished.

## Provenance

- Base: upstream Horizon at commit `962f5949ac1a8f789227614a75bcf360d0fb12a2`
  (pins Minecraft 26.3: protocol 777, data version 5023).
- This version pins Minecraft 26.2: protocol 776, data version 4903,
  resource pack 88.0, data pack 107.1, Java 25.
- Steps run: `prepare 26.3`, `prepare 26.2` (download path),
  `diff 26.3 26.2` (651 ranked work items: rank 1: 7, rank 2: 410,
  rank 3: 177, rank 4: 55, rank 5: 2), `regen 26.2`, oracle adaptations,
  `harnesses 26.2` over the 17-harness port set, then the completion
  pass described below.
- No Mojang-derived material is committed: no jars, no mappings, no
  decompiled sources, no prepared data. The version store and the
  candidate's `.local` trees are local only.

## Regeneration

All six generators regenerated for 26.2 (upstream's own backward run of
this port regenerated three; the first pass here regenerated five):

| generator | result |
| --- | --- |
| version_consts | regenerated (version.rs +8 −8) |
| packet_ids | regenerated (ids.rs +134 −142) |
| entity_tracking | regenerated (tracking.rs +16 −21) |
| lan_command_order | regenerated (command_order.rs +45 −302) |
| command_suggestions | regenerated (command_suggestions.rs +11 −22) |
| vanilla_command_order | regenerated in the completion pass (vanilla_order.rs +5 −24: the 26.3-only `posteffect` and `compute` commands and the dedicated-server segment leave the client tree) |

## Completion pass (this landing)

### Protocol: the three 26.3-only clientbound packets are rewired

`horizon-protocol`'s eight compile errors came from `SWING_ANIMATION`,
`POST_EFFECTS` and `ADD_TRANSIENT_BLOCK`, which have no ids in 26.2's
protocol (776). Each was established from the 26.2 jar and rewired to
26.2's own form, mirroring the pattern the 26.1 completion pass landed:

- **Swings.** 26.2 has no swing packet; `ClientboundAnimatePacket`
  carries swings as action 0 (main hand) and 3 (off hand), next to
  2 (wake up), 4 (critical hit) and 5 (magic critical hit) — verified
  against the jar's `handleAnimate` and `LivingEntity.swing`, which
  builds the animate packet itself. The `SwingAnimation` variant and
  its payload are removed; `game/remote.rs` handles actions 0/3 ahead
  of the general animate arm (local player and mobs swing with the
  default animation state), the LAN host broadcasts `Animate`, the
  session recorder in `game/hand.rs` records `Animate`, and both
  cross-version adapters forward the serverbound hand byte instead of
  synthesising or dropping it. The serverbound rename Punch → Swing
  (play serverbound 63) was already done in the first pass.
- **Post effects.** 26.2 has no `/posteffect` command and no packet
  for server-requested effect chains; post effects are client-local.
  The `PostEffects` variant, its receive arm and the LAN hub's sends
  are removed. The local state stays, inert: `Game.post_effects`,
  `requested_post_effects()` and the guest-side saved fields are kept
  but nothing transmits them.
- **Transient blocks.** The `AddTransientBlock` variant, its receive
  arm and the falling-block broadcast are removed: in 26.2 a landed
  falling block arrives as the plain block update. The client-side
  transient-block module stays, inert — nothing feeds it.
- **Spectator action needed no rewire.** 26.2's
  `ServerboundSpectatorActionPacket` is a record with a single
  `OptionalInt spectateEntityId` (javap-verified); the tree already
  modelled exactly that, and the hub honours it.

`cargo check --workspace --all-targets` is clean (0 errors).
`cargo test -p horizon-protocol`: **436 passed, 2 failed, 7 ignored**.
The 2 failures are sandbox-network environment tests (DNS resolver and
connection tests in untouched code; the sandbox intercepts their
sockets). The 7 ignored are the adapter-ladder debt below.

### The adapter ladder is still keyed to the previous pin

The cross-version adapter's per-version tables are generated against
the *old* pin, and regenerating one version's tables in isolation
breaks the ladder's internal consistency (measured: 8 tests fixed, 23
broken). A full re-base needs every version's prepared reports, and
`prepare` refuses obfuscated releases — the same root cause the 26.1
pass documented for its 32 residual failures; here it is smaller
because 26.2 sits next to the old pin. Seven tests are marked
`#[ignore = "adapter ladder still keyed to the previous pin"]` as the
re-base specification: two session-translation tests, two legacy item
slot conversions (the converter's component table is old-pin-keyed:
component 76 is `minecraft:block_state` in 26.2,
`minecraft:pot_decorations` in 26.3), two jigsaw/pack-response steps
and one login/respawn dimension resolution. Two packet fixtures were
corrected to 26.2's real ids from its protocol report
(`waypoint` 141 → 138, `update_attributes` 134 → 131), and the sign
datafix oracle was re-based to the pinned data version 4903: the
`allow_op_features` fix is keyed at data version 5002, after 26.2, so
the chain correctly omits it. In production paths the staleness is
mostly unreachable: `ServerVersion::from_protocol(776)` resolves to
the current version, so real 26.2 peers get no session translation.

### Adapter-ladder re-base (behaviour/26.2, 2026-11-13)

The ladder is now re-based to the 776 pin. `cargo test
-p horizon-protocol`: **443 passed, 2 failed, 0 ignored** — the 2
failures are the same sandbox-network pair; the 7 ignored tests are
un-ignored and passing. Method: regenerate all 14 registry-id tables
and all 13 packet-id tables from the prepared reports
(`tools/regenerate_registry_id_adapters.py`,
`tools/generate_packet_id_adapter.py`), then re-key the production
conversions and fixtures that baked in the old pin's ids, following
the 26.1 re-base (commit ff7d657) and the 1.21.11 re-base (2e2ef22).

Key structural fact: **26.2's data-component registry is 26.1's with
`minecraft:sulfur_cube_content` inserted at 78**, all ids ≥78 shifted
by +1 (26.2: 111 components, top id 110; 26.1: 110). The 26.1 tree
served as the re-key template for most files: take its file, shift
component ids ≥78 by +1, add sulfur where 26.2 has it.

What changed, by cluster:

- **Tables.** 13 packet tables + 13 registry id tables + `pinned_ids.rs`
  regenerated. Packet renames fed to the generator: `horse_screen_open
  → mount_screen_open` (all versions ≤1.21.10), `debug_sample_subscription
  → debug_subscription_request` (1.20.6–1.21.8), `spectate_entity →
  spectator_action` (26.1.2), `game_profile → login_finished` (1.20.2,
  1.20.3, 1.20.6, 1.21.1 — the rename landed between 1.21.1 and
  1.21.3; missing it left the 765 login test's session stuck in Login,
  because the login-success packet mapped to None).
- **Item component constants and codecs** (`item.rs`, `skip.rs`,
  `hash/raw.rs`, `hash/gameplay.rs`, `hash.rs`): all component ids
  re-keyed to 26.2's registry (diamond 926, stick 974, brick 1054,
  angler sherd 1477); `skip_dropped` now drops only
  `fire_resistant`/`hide_tooltip`/`hide_additional_tooltip` — the only
  components any ladder server sends that 26.2 lacks. The 26.3-only
  components (sign text, waxed, cushion/color, villager_food,
  compostable, cooking/brewing fuel, mob_visibility) are gone.
- **Session translation** (`client_session.rs`, `world.rs`,
  `translate.rs`): the 26.3 PLAYER_ACTION up-shift is removed (the pin
  IS 26.2; every client's actions 0–6 cross unchanged, and 26.2's own
  STAB at 7 is dropped for older servers, never shifted);
  `accept_teleport` returns the id alone.
- **`AcceptTeleportation` rewired** (multi-crate): the pinned packet
  struct still carried the 26.3 layout (id + position + yaw/pitch);
  26.2's packet is the id alone. Fixed in `packets/play.rs`,
  `remote.rs`, `hub.rs`, `bots/main.rs`, `probe.rs` and `lan_bot.rs`
  (6 send sites). 1.21.11's tree has the same latent struct bug, not
  fixed there.
- **Particle option dispatch** (`entity_data.rs`): `Particle::read`
  was keyed to 26.3's particle registry; re-keyed by type name to
  26.2's (entity_effect 28, vibration 55, item 54, trail 56, shriek
  112, dust_pillar 118, block_crumble 122, ...).
- **Legacy item readers** (`item/v1_20_3.rs`, `v1_20_5.rs`,
  `v1_21_1.rs`, `v1_21_4.rs`, `v1_21_5.rs`): the 1.20.3 NBT
  converter's component table and registry context re-keyed; the
  1.21.4 tooltip-flag lists corrected (the 26.3 list read a tooltip
  bool after map_color, misaligning every following read — the root
  cause of the enchantment, pots and tooltip failures); the default
  shearing sound holder 1448 → 1444 (snip is 1443 in 26.2). The 144
  pinned fixture hexes were regenerated through the conversion itself
  and cross-checked byte by byte against the 26.1/26.3 fixtures and
  the registry reports: every difference is an explainable registry-id
  change or a harmless component reordering the tests canonicalize.
- **The 7 ignored tests**: the creative-slot test (client_session) and
  the two 1.20.3 slot tests un-ignored and passing as re-keyed; the
  pack-response and jigsaw tests pass as written; the 765
  login/respawn test passes once the `game_profile` rename is in the
  tables; the adapter test's premise was void for a 26.2 server, so it
  is rewritten as
  `serverbound_packets_1_21_11_lacks_are_not_sent` (1.21.11 has no
  attack packet; resource_pack 49→48, swing 63→60, keep_alive 28→27).

Substitutes fact: the three versions closest to the pin (26.1.2,
1.21.11, 26.2) have `Substitutes::NONE` — for a 26.1/1.21.11 client,
an unmapped pinned block state becomes air, an unmapped item an empty
slot, an unmapped entity is dropped. 26.2's additions vs 26.1.2: 28
blocks, 31 items, 1 entity (sulfur_cube 130) — the sulfur/cinnabar
family.

### horizon-prepare completes (prepare: ok)

The exporter bundle compiles against 26.2 and every exporter runs;
`horizon-prepare` reports `Prepared … Done`, and the pipeline records
**prepare: ok**. The five bundle files from the first pass's list:

- `VanillaChunkGenerationOracle` (also the chunk_generation harness's
  oracle): 26.2 splits `ChunkStatus.TERRAIN` into NOISE → SURFACE →
  CARVERS; the terrain phase is re-staged onto `fillFromNoise`,
  `buildSurface` and `applyCarvers` (with the carving-mask filter),
  heightmaps primed after carving, persisted status CARVERS. The
  HORIZON_FLAT variant's flat-world `RandomState` factory is 26.3-only
  and still throws rather than mis-generating.
- `VanillaBlockLandingOracle`: 26.2 has no `Block.bounceOn` and no
  `updateEntityMovementAfterFallOn` anywhere (constant-pool scan of
  the jar: no hits) — bouncing lives inside `fallOn` with
  `Block.getBounceRestitution()`. The oracle's bounce axis is dropped
  (the export's `bounce` field is false for every block; the Rust
  consumer's shelf-mushroom path stays inert), the shelf-mushroom
  bounce sound comparison is removed, the fallOn declarer set gains
  `BedBlock` (which declares `fallOn` in 26.2), `FarmlandBlock`'s
  missing `baseBlock` field falls back to dirt (26.2's `turnToDirt`
  hardcodes it), and the `Level` stub gains the abstract `fuelValues()`
  and `potionBrewing()` overrides 26.2 requires.
- `VanillaMovementRegistryOracle`: `Block.getFallDistanceReduction()`
  and the `fallDistanceReduction` field are both 26.3-only. The export
  now guards the reflective field lookup and writes the neutral 0.0f,
  which is exact for 26.2 (the Rust consumer computes
  `distance * (1.0 - reduction)` and no 26.2 block carries a reduction).
- `VanillaCuboidModelOracle`: `CuboidModelElement.shade()` is a
  boolean in 26.2 (26.3's `shadeDirectionOverride()` is the rework);
  the export reads `shade()` directly.
- `VanillaSimpleBlockOracle` and `VanillaPlacementOracle`: the
  pre-restructure worldgen API. Features are `ConfiguredFeature`s
  parsed through `ConfiguredFeature.CODEC` / `DIRECT_CODEC` and placed
  via `FeaturePlaceContext`; block state providers are not a registry
  in 26.2 (no `Registries.BLOCK_STATE_PROVIDER`, no `DIRECT_CODEC`) —
  catalog entries are parsed through `BlockStateProvider.CODEC` and
  validated inline; `RegistryOps.RegistryInfoLookup` returns
  `RegistryInfo`; placement modifiers are read through
  `getPositions(...)` and the protected `RepeatingPlacement.count` is
  invoked reflectively; the noise settings are built from
  `NoiseSettings(min, height, 1, 2)`, `NoiseRouterData.none()` and
  `SurfaceRules.state(...)` (26.3's `densityfunction` and
  `material.rule` packages do not exist in 26.2).

Two prepare defects outside the bundle were fixed on the way:

- `VanillaCommandParseOracle.exportRegistries` looked recipes up as a
  registry (`lookupOrThrow(Registries.RECIPE)`), which 26.2 does not
  provide — this was also the `command_parse` harness's runtime
  failure in the first pass. Recipes live in the `RecipeManager` in
  26.2; the export reads them from
  `ReloadableServerResources.getRecipeManager()`.
- horizon-prepare's worldgen bundle check demanded non-empty bundles
  in both jars. In 26.2 the material rules, material conditions and
  block state providers are code-only — neither jar carries
  `worldgen/material_rule`, `worldgen/material_condition` or
  `worldgen/block_state_provider` resources — so the check now
  requires the two jars to *agree*, and permits an empty bundle.

## Harness profile

First-pass profile (the baseline this pass had to beat):
**prepare failed; pass 1 (`food`); oracle 1 (`chunk_generation`);
fail 15.**

This pass, verified locally:

- **prepare: ok** (see above — the exporter bundle was the baseline's
  blocker).
- **`chunk_generation`'s oracle compiles again** (javac against the
  26.2 server classpath, exit 0), so the baseline's only
  oracle-classified harness is cured at the source.
- The remaining harnesses were **not re-run to verdicts on this
  machine**: the probe build for the subset exceeded the local window
  (a shared 1.5-CPU cell under several agents' parallel builds — one
  crate compile drew ~20% CPU for over half an hour — and the VM
  rebooted once mid-run, killing the first attempt). The full
  17-harness profile runs on the port-verify CI workflow on GitHub
  runners instead, the same split the 26.1 completion pass used.
  Two baseline failure causes are removed by construction and noted
  here rather than re-measured: `hotbar`, `lan`, `options` and
  `resource_pack` failed because `client_probe` could not build until
  the protocol compiled (it now does), and `command_parse` failed on
  the recipe lookup fixed above; `sound_event` stopped on a
  prepare-gated resource (prepare now produces it).
- Baseline per-harness notes that still stand: `command_exec` hit the
  600 s harness timeout in this environment (oracle compiles clean);
  `datafix`'s sample-world generator downloads a historical server jar
  per version (38 versions) and those downloads hang from this
  environment — environmental, on top of the real DataFixer work item
  below; `entity_components`' driver raises a TypeError on 26.2's
  entity-component data shape; `light`'s oracle authlib key fetch
  timed out from this environment; `save_roundtrip`'s vanilla login
  timed out on both attempts (server boot under load).

Upstream's own backward run of this port (their published status)
ended at 1 pass, 4 oracle-compile failures (chunk_generation,
command_exec, command_parse, hotbar) and 12 fails, with prepare
blocked earlier, at VanillaHeightmapOracle.

## Adaptations already made (first pass)

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
  clean against 26.2.
- **Chunk generation oracle, second pass**: `RandomState.create` argument
  order and `ChunkGeneratorStructureState.createForNormal` without the
  `ChunkPos` argument; the 26.3-only uncached biome resolver hooks are
  marked rather than faked, and the HORIZON_FLAT variant (whose flat-world
  `RandomState` factory is 26.3-only) now throws instead of silently
  mis-generating.
- **Serverbound arm-swing packet renamed** Punch → Swing in the protocol enum
  and every sender, handler and adapter reference (`SWING`, play serverbound
  63).

## What remains, by cause

### Harness oracles that need a rework (outside the prepare bundle)

- first_person: 26.3 extracted hand state into
  `LocalPlayer.firstPersonHandsAndItems` and swing state into
  `LivingEntity.SwingState`; in 26.2 the heights live on `ItemInHandRenderer`
  and swings are the `swinging` / `swingTime` fields. (26.2's item data
  has exactly one swing component, `minecraft:swing_animation` with
  type NONE/WHACK/STAB — there are no separate attack/interact
  animation components.)
- item_model: 26.3's model rework (`ResourceManager.Selector`,
  `ItemQuads`, `MaterialBaker`, `BundleContents.asMutable`,
  `VanillaRegistries.createWorldLookup`) — the oracle needs re-derivation
  against 26.2's ModelBakery API. The cuboid-model exporter's boolean
  `shade` is fixed (above); whether the fixture's model consumer needs
  the same adaptation is for the harness's next run to show.
- knockback client: `ServerboundSwingPacket` has no `INSTANCE` singleton in
  26.2 (it is constructed with the hand), and
  `ServerboundAcceptTeleportationPacket` gains a parameter.
- game_loop: `LocalPlayer` constructor, `MouseHandler.onMove` / `onScroll`
  access, and the input `Type.KEYBOARD` constant all differ.
- furnace: `VanillaFurnaceOracle` still reflects 26.3's
  `AbstractFurnaceBlockEntity.speedMultiplier` (set + read) and the
  driver still sends/expects a SPEED field; 26.2 has no fuel speed.
  The Rust side is ported (rank-2 item 154) and the fixture accepts
  and ignores SPEED, but the harness cannot run against 26.2 until
  the oracle + driver drop speed.
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

### Behaviour work from the diff (rank 1 closed 2026-11-13)

Rank 1 (7/7) closed and merged to main (e9980f99, branch
behaviour/26.2). The largest item, #1 components (+2/−13 data component
types), is ported and verified: 111-component table, `map_color` tint,
furnace fuel from `FuelValues`, tag-based brewing fuel (20 uses, 400
ticks), the 115-entry compost table, sign text/wax dropped on item form
and across the wire, hardcoded villager food and mob visibility
(piglin brutes are NOT fooled), 23 hardcoded pottery patterns, and the
block transformer rebuilt from 26.2 tables (strip 23, scrape 45
unwaxed-only, wax-off 60, till, flatten, dowse) — block-transformer
parity 70932/70932. Two supporting fixes: facts.json accepts the 26.2
NBT-form block-state codec, and the 26.3-only motion tags were replaced
with per-state `blocksMotion()` (`isSolid` except cobweb/bamboo
sapling; 27,816 of 32,366 states). Rank 2 (410 items) is next, in
batches of ~50, triaged by intersecting each item's named methods with
the actually-changed methods in the store's diff.json.

## CI profile — port-verify v5, run 43086869031 (2026-11-11)

The full 17-harness profile the completion pass deferred to CI is now
measured on GitHub runners, against the completed tree (candidate commit
9b4a521d1f92, main@2596a39 incl. 2d06bb3). `prepare: ok`; horizon-prepare
completed in 25.1 s; protocol compiles clean (0 errors — the 8-error
blocker is gone).

**Profile: 3 pass / 0 oracle-no-longer-compiles / 14 fail.**
Passes: food, lan, resource_pack. (The "14 newly failing" count is a
no-baseline classification artifact — no baseline was passed to the run.)
Against the codespace verification of the pre-completion tree (1 pass /
1 oracle / 15 fail), the oracle failure is cured and two more harnesses
pass; against completed 26.1's CI profile (3 pass / 0 oracle / 14 fail)
the count matches but the pass set differs (26.1 passes block_properties,
26.2 passes resource_pack instead). Sampled failure causes, all real
behaviour/oracle mismatches now that the probe builds and runs:
chunk_generation's oracle crashes at runtime (`IllegalStateException:
Requested chunk unavailable during world generation` in the
Beardifier/structure path); command_parse runs to comparison
(34,582/34,583 commands match at permission 0) but reports 6,266 parse
mismatches, with the Rust suggestion list still offering `compute` where
vanilla does not; block_properties' world_probe fixture exits 1. These
are the next behaviour/oracle work for this tree, not verification gaps.
Artifacts kept at `~/workspace/mcir-ci/26.2v5-completed/`.

## CI profile — port-verify v6.6, runs 43342783340 / 43352941442 (2026-11-13)

Pre-behaviour baseline, measured twice with the same result: **3 pass /
0 oracle / 14 fail** (food, lan, resource_pack pass; protocol tests
438/0 on the runner). Run 43342783340 on main@b16d58da; run 43352941442
on behaviour/26.2@e8310c4f (post-ladder, pre-behaviour). Artifacts for
43352941442 kept at `~/workspace/mcir-work/26.2-behaviour/ci-runs/43352941442/`.
Harness-log finding from 43352941442: command_exec, light and
save_roundtrip failed with the facts.json `missing field 'id'` parse
error — fixed on behaviour/26.2 since (NBT-form codec accepted), so the
next profile should move.

## CI profile — port-verify v6.6, run 43379593930 (2026-11-13)

Dispatched on behaviour/26.2@42cb47fe (rank 1 closed, 7/7; rank 2 at
0/410 — batches 1–5 landed after the dispatch, in bccbf9ac..48b3fe71):
the first profile of the behaviour-ported tree. Completed
2026-11-13T08:39:43Z, conclusion success (the harness leg is
continue-on-error, so the conclusion is not the profile).

**3 pass / 0 oracle / 14 fail** — identical to the v6.6 baseline on main
and to run 43352941442: food, lan, resource_pack pass; the other 14
fail. D1 pass, D2 pass (0 errors, 0 warnings; protocol tests 445, none
failing — the 7 un-ignored tests from the re-base are in); D3 fail
(rank 2: 410/410 neither closed nor accepted at this commit); D4–D6
blocked as before. The rank-1 work moved no harness, as expected at
this stage. The facts.json `missing field 'id'` parse error seen in
43352941442 did not recur in this run's harness logs (the NBT-form
codec fix is in), but the affected harnesses still fail on their own
differentials. Artifacts kept at
`~/workspace/mcir-work/26.2-behaviour/ci-runs/43379593930/`.

## CI profile — port-verify v6.6, run 43405337919 (2026-11-13)

Dispatched on behaviour/26.2@afe697ed (the occlusion.nbt
`BlockState::from_nbt` fix; rank 2 at 65/410 — item 104 landed after
the dispatch, in e345f3ce). Completed 2026-11-13T11:59:31Z, conclusion
success (the harness leg is continue-on-error, so the conclusion is
not the profile).

**4 pass / 0 oracle / 13 fail** — food, lan, **light**, resource_pack
pass; the other 13 fail. **Light moved from fail to pass**: the
`from_nbt` fix unblocked the launch crash (`NBT at byte 1975826:
invalid table state`), so the light harness finally runs. command_exec
and save_roundtrip still fail — save_roundtrip has the separate
vanilla-side scene-command serialization errors (decorated_pot sherds,
BlockState "No key Name") noted with the fix, and both need behaviour
work, not just launch. D1 pass, D2 pass (0 errors, 0 warnings;
protocol tests 445, none failing); D3 fail (rank 2: 345/410 neither
closed nor accepted at this commit); D4–D6 blocked as before.
Artifacts kept at
`~/workspace/mcir-work/26.2-behaviour/ci-runs/43405337919/`.

## CI profile — port-verify v6.6, run 43435321313 (2026-11-13)

Dispatched on behaviour/26.2@576d4a93 (the BedBlock landing-family
fix; rank 2 at 66/410). Completed 2026-11-13T15:46:04Z, conclusion
success (the harness leg is continue-on-error, so the conclusion is
not the profile).

**4 pass / 0 oracle / 13 fail** — food, lan, light, resource_pack
pass; the other 13 fail. The BedBlock fix worked: no "unsupported
landing family" in the logs, and the occlusion crash stays gone. But
command_exec and save_roundtrip still fail on a NEW unknown launch
crash: the client exits 1 at launch before opening the control socket,
and the crash report stayed on the runner, so its cause is not in the
artifacts. Diagnosing it by local reproduction is a second track
alongside the rank-2 port. save_roundtrip separately still shows the
vanilla-side scene-command serialization errors (decorated_pot sherds,
BlockState "No key Name"). D1 pass, D2 pass (0 errors, 0 warnings;
protocol tests 445, none failing); D3 fail (rank 2: 344/410 neither
closed nor accepted at this commit); D4–D6 blocked as before.
Artifacts kept at `~/workspace/mcir-ci/26.2-bedfix/`.

## CI profile — port-verify v6.6, run 43519358480 (2026-11-13–14)

Dispatched on behaviour/26.2@7379b0a5 (rank 2 at 151/410 — items
19–21 landed last). Completed 2026-11-14T00:31:23Z, conclusion
success (the harness leg is continue-on-error, so the conclusion is
not the profile).

**4 pass / 0 oracle / 13 fail** — food, lan, light, resource_pack
pass; the other 13 fail. Unchanged from run 43435321313: the rank-2
behaviour items landed so far move no harness in the 17-harness
subset, as expected — the launch crash still blocks command_exec and
save_roundtrip, and the other failures are oracle compiles, the
missing sounds step, or behaviour not yet ported. D1 pass, D2 pass
(0 errors, 0 warnings; protocol tests 445, none failing); D3 fail
(rank 2: 259/410 neither closed nor accepted at this commit); D4–D6
blocked as before. Artifacts kept at
`~/workspace/mcir-ci/26.2-151/`.

## Rate sample — changed vs no-change (Freebuff, from 2026-11-14)

Over the next 100 rank-2 items closed starting at 218/410 (items
219–318 of rank 2), record the split: CHANGED (needed a real port) vs
NO-CHANGE (verified against the jars with no code change). Time ONLY
the changed items, in wall-clock minutes. The sample decides whether
the 874 delta port-days figure is right or ~3× too high, so the
changed/no-change split and the minutes on changed items are the
numbers that matter; no-change verifications are counted, not timed.

**rate sample: 6/100 items, 3 changed, 3 no-change, 3 minutes on changed
items.** Sample starts at rank 2 218/410 (2026-11-14 ~07:56 NPT).
Items 88 and 231 were in flight when the sample started; their work
predates it, so they count as changed with minutes "pre-sample"
(excluded from the minutes total). Item 89 was already closed before
the sample started, so it is not counted. Timing starts with the first
item begun after the sample started. The table below is the running
log; it stays in agreement with the tally line above.

| item | changed? | minutes | note |
| --- | --- | --- | --- |
| 88 | changed | pre-sample | worldgen settings port, stages 1–6 |
| 231 | changed | pre-sample | carver port (cave/canyon/nether, apply phase) |
| 133 | no-change | — | 26.3 densityfunction/op package absent from 26.2 |
| 152 | no-change | — | 4 util classes 26.3-only, no Rust references |
| 153 | no-change | — | CarverOutput/Filter/Visitor 26.3-only |
| 102 | changed | 3 | bed pair check removed (26.2 form), BedRule explodes alias |

## CI profile — port-verify v6.6, run 43535967019 (2026-11-14)

Dispatched on behaviour/26.2@bc530de6 (rank 2 at 218/410). Completed
2026-11-14T02:48:03Z, conclusion success (the harness leg is
continue-on-error, so the conclusion is not the profile).

**4 pass / 0 oracle / 13 fail** — food, lan, light, resource_pack
pass; the other 13 fail. Unchanged from run 43519358480: the rank-2
behaviour items landed so far move no harness in the 17-harness
subset, as expected — the launch crash still blocks command_exec and
save_roundtrip, and the other failures are oracle compiles, the
missing sounds step, or behaviour not yet ported. D1 pass, D2 pass
(0 errors, 0 warnings; protocol tests 445, none failing); D3 fail
(rank 2: 192/410 neither closed nor accepted at this commit); D4–D6
blocked as before. Artifacts kept at
`~/workspace/mcir-ci/26.2-218/`.
