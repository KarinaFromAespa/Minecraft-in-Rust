# Port notes: Minecraft 26.2

This directory is the 26.2 port produced by the version-port pipeline
(`tools/version_port.py`, see `docs/version-ports.md`), ported backward from
the programme's 26.3 base. The completion pass has landed: the protocol
rewire is done, all six generated tables are 26.2's, horizon-prepare runs
end to end, and the workspace compiles clean. What remains is itemised
below with its current state. Nothing in this file is a claim that the
port is finished.

## Pin location

26.2 is the first release whose pin metadata moved out of the tree under the
lines/+pins/ restructure (2026-11-16): `work-items.json`, `status.json`,
`tier.txt`, `harness-subset.txt`, `regen-determinism.json`, `rate-sample.json`,
the `rank*-progress.json` ledgers, the `diff-26.3-26.2/` report, `profile.json`
and `delta-verdict.json` all live in `pins/26.2/`. `research-lock.json` stays
in this tree — the gate, the workflow and `version_port.py prepare` all read
it here. This file stays here too: it is the port's narrative, not gate input.

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

**rate sample: 108 items COMPLETE, 47 changed, 61 no-change, 4162 minutes on changed
items.** Sample starts at rank 2 218/410 (2026-11-14 ~07:56 NPT) and closes at rank 2
326/410 (2026-11-15). The last two closes were 322 (changed, 30 min est.) and the
nine no-change items of batch 4. Correction 2026-11-15: the tally line originally
read 100/100 with 53 no-change; the table holds 108 rows — the 218→326 window spans
108 closures — with 61 no-change. The changed count (47) and minutes (4162) verify
exactly against the table. Machine-readable form: pins/26.2/rate-sample.json.
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
| 110 | changed | 6 | Blender to f64 (BlendingOutput, blend_density, blend_biome, OutputBuffer) |
| 105 | changed | 35 | environment colors to 26.2 packed-int formulas; 4 oracles + 4 harnesses pass |
| 135 | changed | 4 | VillagerTrade reads given_item_modifiers (plural) |
| 136 | no-change | — | tryPlaceLeaf plumbing only; Rust already matches 26.2 |
| 134 | changed | 3 | port-pack gating: 26.2 series copies as-is, 26.3 packs not brought over; install() fresh-dir fix |
| 140 | changed | 11 | container tooltip args (name, count); crossbow multiple line uses getDisplayName |
| 119 | changed | 6 | onMove to 26.2 absolute-delta form; grab centre integer division |
| 99 | no-change | — | crumbling already the 26.2 parameter form; rest is API-only or has no Rust counterpart |
| 90 | changed | 11 | VideoMode to 26.2 int-rate string form; fullscreen no-adopt, exclusive at construction; throttle iconified-only; FifoRelaxed vsync |
| 94 | changed | 28 | Animate codes to 26.2 numbering; post-effect system removed; /publish gamemode; sleep early-refusal order (includes a compaction + service-restart pause) |
| 355 | no-change | — | BrewingRecipe 26.3-only; 26.2 brewing is PotionBrewing mixes; book arms unreachable-with-effect |
| 356 | no-change | — | Rust DensitySampler is Horizon's own trait; RangeChoice/IntervalSelect semantics verified vs the 26.2 jar |
| 393 | no-change | — | Rust DensityVolume is Horizon's own volume index math, not a port of the 26.3 class |
| 397 | no-change | — | InputQuirks$1 is the synthetic switch-map for 26.3's keyboard display overrides; Rust has none |
| 407 | no-change | — | ClockInstance 26.3-only; Rust already uses 26.2 ClockManager.getTotalTicks (item 105) |
| 267 | changed | 14 | classes table gains DirtPathBlock + RedStoneWireBlock aliases (both classes never matched in 26.2); survival tables updated |
| 247 | changed | 6 | eyeblossom neighbour scan to 26.2 betweenClosed order (x, y, z), not 26.3 section-major |
| 177 | changed | 120 | brewing to 26.2 PotionBrewing mixes (65) + BrewOutput model; stand to flat 26.2 form (no totals/speed); both harnesses reworked (5,386 + 20 lines match); shared block-update oracle to 26.2 (blockscan/ClockInstance/ServerChunkCache); merchant float-number fix; spans the investigation (pre-compaction) + implementation |
| 238 | changed | 10 (est.) | lerp + distance_to_point decoder arms removed (26.2 bootstrap registers neither); 18 density-compiler classes 26.3-only |
| 357 | no-change | — | Rust Noise trait / NoiseStack is Horizon's own engine; Interval mentions are comments on float edge-cases; range semantics belong to item 127 |
| 361 | changed | 20 (est.) | minecraft:ore_vein decoder arm removed (26.2 MATERIAL_RULE has only bandlands/block/sequence/condition); Definition::Ore + roots plumbing removed; MaterialRule::OreVein stays for the generator's 26.2 vein application |
| 362 | no-change | — | ClayBands matches 26.2 SurfaceSystem.generateBands/makeBands exactly (verified line by line); comment reworded to name SurfaceSystem |
| 182 | no-change | — | RandomState$2 is a 26.3-only anonymous DensitySamplerSet impl; Rust SeededContext is Horizon's own holder |
| 191 | no-change | — | ItemContainerContents$Mutable, TooltipProvider$Getter 26.3-only; Rust mentions are comments |
| 199 | no-change | — | DynamicGpuData + inners, GameRenderer$1, LevelRenderer$ChunkDrawGroup absent from 26.2 client jar; no Rust references |
| 208 | no-change | — | SignBlockEntity$1, SignText$Mutable 26.3-only; no Rust references |
| 228 | changed | 20 (est.) | minecraft:gradient decoder arm removed (26.2 bootstrap registers only y_clamped_gradient); 22 generator classes 26.3-only |
| 230 | no-change | — | VanillaPackResources$1 26.3-only; only Rust mention is a comment |
| 232 | no-change | — | material/condition/* is a pure rename of SurfaceRules conditions; both registries register the identical 11 types |
| 235 | no-change | — | BrewingFuel/CookingFuel/ResolvableFloat/ResolvableInt 26.3-only; Rust component table verified against 26.2 DataComponents order |
| 240 | changed | 90 (est.) | Straw beds removed (26.3-only): flat 0.6875 setPosToBed, no sleep-height check, no destroy-on-leave, no straw_bed stat/sprite/POI |
| 241 | changed | 90 (est.) | SwingState removed: 26.2 plain-field swing model (swingTime/attackAnim, live getCurrentSwingDuration); oracle reworked |
| 242 | changed | 180 (est.) | Placement modifiers to 26.2: random_offset replaces offset/cuboid/random_chance/randomly_selected; FeaturePlacer→PlacedFeature.place; 13 oracles + 11 drivers reworked; offsetCodec strict-<16 bound; height_range/volume_match predicates removed (26.3-only). Placement harness 1,674→882/18,780 mismatches |
| 276 | no-change | — | BrewingInput 26.3-only; zero Rust references |
| 180 | changed | 60 (est.) | cooking defaults to 26.2 (smelting 200, blasting/smoking/campfire 100); TransmuteResult 26.3-only, Rust Template already the 26.2 form |
| 185 | changed | 45 (est.) | recursive predicate cycles are warnings-only in 26.2 (no hasFatalProblems throw); only Unbound refuses the start |
| 187 | no-change | — | DecoratedPot/ChestRenderer submit plumbing only; animation state unchanged; BlockTags additions all 26.3-only |
| 205 | no-change | — | VillagerFood 26.3-only; Rust already the 26.2 FOOD_POINTS form; take_half_stack exact |
| 296 | no-change | — | TrimTextureKey$1 26.3-only; CustomHeadLayer Transforms 5 vs 3 fields, Rust mentions are comments |
| 302 | changed | 90 (est.) | dolphin goals merged to the single 26.2 PlayWithItemsGoal (ItemGoal/MoveToItemGoal do not exist in 26.2); Jump goal to blocksMotion(); 2-arg moveTo reach 1 |
| 305 | changed | 60 (est.) | Panic water search to withinManhattan order (depth shells, x, y, +z/-z); MeleeAttack tick reach 1 (2-arg moveTo) |
| 306 | no-change | — | StreamCodec$20/$21 26.3-only; zero Rust references |
| 307 | no-change | — | BlockTransformer 26.3-only; block_transform.rs already 26.2 |
| 311 | no-change | — | levelgen/material/rule/* 26.3-only; 26.2 uses SurfaceRules |
| 316 | changed | 120 (est.) | enderman teleport to the 26.2 two-scan blocksMotion form (no #entities_can_teleport_to, no avoidance tag, no border clamp, no level event 2018); EnderMan rename; 64-try loop inline |
| 321 | no-change | — | MaterialRuleContext 26.3-only; 26.2 uses SurfaceRules |
| 333 | no-change | — | level/blockscan package 26.3-only; cushion oracle's blockscan import belongs to items 147/374 |
| 342 | no-change | — | OIT entirely 26.3-only; Rust OIT mentions are comments + opt-in tagging |
| 233 | changed | 420 (est.) | EntityPositionSync to PositionMoveRotation; VecDelta to [i16;3]; MoveEntity 26.2 wire forms; AdvancementDisplay x/y; sign bool side; InterpolationHandler rework (double alpha, bitwise angle compare); tracker reworked; harness bit-exact on 135,742 lines. Outlier: multi-crate rework + harness |
| 192 | changed | 60 (est.) | powder-snow entityInside to the 26.2 client-side form (xOld/zOld gate, addParticle); interpolation half closes with 233 (minutes counted there) |
| 157 | changed | 120 (est.) | EntityFluidInteraction to the 26.2 per-tag Tracker model (joint rework with 161): void update, plain-top eye check, tracker-height flow scaling, untagged fluids skipped; getHeightForCamera removed from the trait + 4 implementors; stale ServerEntity/UpdateInterval comments fixed |
| 161 | changed | 120 (est.) | EntityFluidInteraction + FloatGoal (joint rework with 157): tick answers inWater \|\| inLava; mob eye check to the plain top; FloatGoal verified already 26.2 (no code change); entity-fluid harness reworked (oracle reflects trackerByFluid, sets the Fluid holder field 26.2's getTrackerFor reads) and bit-exact on 560,630 lines against both 26.2 jars |
| 117 | changed | 90 (est.) | TreeGrower to the 26.2 secondary-chance model (oak 0.1, spruce 0.5, mangrove 0.85); poplar removed; canGrow removed (lone dark oak takes the meal); mega square cleared to air/260, single sapling to legacy block/260; MushroomBlock verified equivalent; 3 picker unit tests |
| 163 | changed | 60 (est.) | Cat goals to the 26.2 forms: can_sit_on hardcoded (chest/furnace/bed-foot, no #cats_can_sit_on tag); CatLieOnBlock renamed CatLieOnBed, checks #beds; RelaxOnOwner bed check to #beds; giveMorningGift to the 26.2 4-arg randomTeleport via a shared helper extracted from enderman.rs (leash-holder origin added); CatRenderer setupRotations and BiomeCheck verified no-change |
| 92 | changed | 90 (est.) | Entity culling to the 26.2 tick-position form (no getInterpolatedBoundingBox): cull_box + standing_entity_in_view drop interpolation, 8 call sites updated; stale SwingState/currentSwing citations fixed (swing model already 26.2 under 241); 26 renderer/layer classes verified no-change (rotate renames, submitModel null arg, render-type plumbing) |
| 87 | changed | 180 (est.) | See-through name tags back to the 26.2 pass position (drawn with the translucent features, before translucent terrain; late draw only in OIT/flat/pack frames); forcefield texture to the 26.2 plain SimpleTexture form (1 mip level, NEAREST min / LINEAR mag, no mip filtering; MipmappedTexture is 26.3-only). LevelRenderer diff is the RenderPearl migration + 26.3-only OIT passes; addSkyPass/submitBlockOutline/waterMask already 26.2; Util.toMillis, DebugCrosshair, gizmos, DepthStencilState, NativeImage$Format verified no-change |
| 101 | changed | 150 (est.) | Loot to the 26.2 list forms: `functions`/`conditions` lists at table, pool, entry and per-function level (26.3's single `modifier`/`condition`); both rollers (loot.rs, items.rs LootContext) + the odds calculator converted. Real gap: 26.2 leaf entries DO carry `functions` (LootPoolSingletonContainer composite, innermost) — loot.rs `create` never applied them; now does, pinned by a new order test (2 + 3 + 10 = 15). NestedLootTable already 26.2 (Either value); function/predicate math identical (plumbing only); Instrument already 26.2; MobVisibility already noted 26.3-only |
| 115 | changed | 120 (est.) | DimensionOrigin placement removed (26.3-only): variant, decode arm, registry `origin` field and the three origin methods gone; a dimension_origin set now decodes to Unsupported and never starts (pinned by a new test). Placement shared logic identical across the 26.3 class/interface split (reducers verified against the enum mapping). Structure biome semantics identical (stub-position check already in generate). Beardifier math identical; float-vs-double precision is density-engine-wide debt, recorded, not reworked |
| 109 | changed | 120 (est.) | GameTest to the 26.2 single-level/single-grid form: TestDef.dimension and Info.dim removed, Runner holds its level, Grid is one state, onBatchStart does NOT reset the row count, say is level-scoped via broadcast_level, summary coordinate links removed (26.3-only) |
| 106 | changed | 90 (est.) | Natural spawning: quart-origin spawn costs; fresh pufferfish 0.35 (puff state 0); obstruction predicate kept (blocksBuilding) |
| 131 | no-change | — | RenderSystem diffs are the SDL/RenderPearl vs GLFW/GLX platform rework; fluid mesher names none of the changed methods; world_border_render oracle needs a 26.3->26.2 rework (26.3-only API) |
| 171 | no-change | — | Biome sampling math identical; diffs are API rework (Sampler arg, double compute, RTree.create, codec renames); spawnTarget/findSpawnPosition is a 26.2-only dedicated-server gap, not a sampling change |
| 214 | changed | 130 (est.) | RuleTest to the 26.2 2-arg form (height/not/all_of/any_of removed); BlockState.CODEC object-only NBT names (Name/Properties); rule_test harness reworked and passing (1,251,780 lines match, both jars) |
| 236 | no-change | — | Core renames/inlines with identical semantics (Holder, Vec3i strict <, Direction, canSerializeIn, MutableBlockPos); Axis per-constant abstract methods |
| 239 | no-change | — | ColorTargetState package move (Blaze3D -> RenderPearl) only |
| 244 | no-change | — | RenderType import move + outline-texture lookup + removed hasBlending; Rust does not use Mojang's RenderType |
| 246 | no-change | — | Item$Properties 26.3-only villagerFood/potPattern + components absent in 26.2; shared methods identical |
| 248 | no-change | — | ContextMap map-implementation swap + method renames (set/get -> withParameter/getParameter); semantics identical |
| 249 | no-change | — | TagSlotDisplay HolderSet (26.3) vs TagKey (26.2); Rust expands tags eagerly at load; SlotDisplayContext adds FUEL_VALUES in 26.2 |
| 250 | changed | 60 (est.) | State-provider states to the 26.2 object-only block-state codec (simple, weighted, noise default/states, would_survive); state_provider harness needs a 26.3->26.2 oracle rework (97 compile errors) |
| 251 | no-change | — | WeatherEffectRenderer is the RenderPearl/OIT rework; Rust has its own wgpu weather renderer |
| 255 | no-change | — | SimpleBlockFeature record+MapCodec (26.3) vs Feature<SimpleBlockConfiguration> (26.2); Rust already decodes to_place directly with optional schedule_tick |
| 256 | no-change | — | StructureCheck cache is a pure optimization; addChildren accessor rename; BoundingBox same values |
| 259 | no-change | — | TagCommand response-plumbing rework; data accessors ArgProvider vs DataProvider; behaviour identical |
| 275 | no-change | — | calculateSolid logic identical (26.3 adds SolidDebugger logging); blocksMotion/isViewBlocking already 26.2-form; shouldRedstoneWireConnectTo 26.3-only, uncalled |
| 277 | changed | 60 (est.) | isSuffocating default to blocksMotion (no causes_suffocation tag in 26.2); BlockState/StateHolder codec already fixed in 214 |
| 297 | no-change | — | SetNameFunction list predicates (item 101); ExplorationMap already the 26.2 TagKey form |
| 298 | no-change | — | TrimMaterial 26.2 assets form (item 61); BannerPattern codec rename |
| 299 | no-change | — | AtlasGlyphProvider import move only |
| 300 | no-change | — | CuboidModelElement shade bool already read (both forms) |
| 301 | no-change | — | MessageSignature buffer API only; wire format identical |
| 303 | no-change | — | LootTable$Builder list form (item 101); LootContext rename |
| 304 | no-change | — | ChunkSectionLayer import move + multidraw variant removed |
| 308 | no-change | — | ServerConnectionListener queue vs direct accept; Rust has its own listener |
| 309 | no-change | — | Sign block MapCodec present in 26.2, removed in 26.3; Rust does not use block codecs |
| 310 | no-change | — | isValidClickButton SDL 1 vs GLFW 0 (same left click); getScrollAmount 26.3-only |
| 129 | changed | 150 (est.) | NarrationTrigger removed (26.3-only): `Trigger` enum, `Output.trigger()`, `ScreenNarration.next_trigger` gone; `schedule`/`due`/`run`/`after_status_change` take no trigger; `ContainerObjectSelectionList` narrates the hovered row whenever one exists (26.2 form) instead of only for mouse/system triggers. Clock comments re-cited to 26.2's `ClockInstance` (26.3's `ServerClockInstance` is the renamed public form; tick/loadFrom/packState identical). Key codes (GLFW vs SDL), `isInputCaptured`, `fillCrashDetails`, openUri and the `post_effects` suggestion provider verified no-change (winit keys, no Rust refs, or string-URI form already 26.2) |
| 8 | changed | 300 (est.) | Swing packet carries the hand (26.2 form); login takes the server previous game type; wall nudge drops the passenger exclusion; F4 flips the spectated shader (`effectActive`); F3+Escape pause keeps the dig; creative entity attack sets no destroy delay; mining fatigue is the 26.2 table (0.3/0.09/0.0027/0.00081). First-person oracle reworked to 26.2 (`ItemInHandRenderer`, 8-arg `LocalPlayer`, singular `addBreakingBlockEffect`, `SoundManager` shell). Harness: 4,205 first-person cases match (382,720 lines) |
| 12 | changed | 300 (est.) | PlayerAction ordinals are 26.3-shifted (CHANGE_DESTROY_DIRECTION at 1): client sends + host dispatch to 26.2 ordinals, face-change send removed; glide start bars water only (no lava arm); piglin-brute visibility + clamp removed; restitution strict `<`; collide query box drops maxUpStep; hurtServer knocks back even fully blocked; forceSendPlayerSupportBlocks (26.3-only) removed; duplicate-position kick (26.3 receivedPositionThisTick) removed. New ordinal regression test passes; health tests 14/14 |
| 86 | changed | 240 (est.) | Farmland trample + turtle-egg destroy drop 26.3's spawn-protection gate (26.2 players always trample/destroy); bucket evaporate smoke to 26.2 addParticle x8 uniform in-cube, acting client only, LAN send removed; spectator block outline over menu blocks; spinning/OIT/post-effect citations to the 26.2 locations. cargo check clean; lan_particles 5/5; block-landing harness block-landing parity harness vs the 26.2 oracle: 5,597,105 registered lines match (client + server) |
| 147 | changed | 120 (est.) | BlockAttachedEntity.kill is Entity.kill(level) in 26.2 (unattributed death); painting/frame ENTITY_DIE now unattributed on player and blast breaks; explosion_kill_credit() + crystal_blast_player() removed; explosion-broken frame BLOCK_CHANGE now attributed to the blast's direct source entity (exploding mob) via explosion_direct_source(). entity_die_explosions updated to the 26.2 truth (4 events, all self-sourced; no item event); stand hurt test renamed + fixed (same 26.3 attribution lag). entity_die 2/2, stand 12/12, kill_die 1/1, vibration 6/6, lan_particles 5/5 |
| 374 | changed | 120 (est.) | Cushion/CushionItem/CushionRenderer 26.3-only (0 entries in either 26.2 jar): entity/item/renderer, dye_index, Target::Cushion, minecraft:cushion load/save, LAN states, seats/rides, blast/thunder handling and the placement probe removed; generic logic moved (dismount helpers to passengers.rs, suffocation/view-blocking to render_tick.rs). cushion_placement harness n/a for 26.2 (oracle imports the 26.3-only blockscan package); driver + oracle left in place |
| 312 | no-change | — | GameType -1 sentinel already the Rust form (item 8); BaseSpawner.setEntityData 26.3-only; SpawnPredicate no ServerLevel param in 26.2 (item 106 form) |
| 313 | no-change | — | RegistryOps/HolderLookupAdapter HolderOwner rework; no Rust surface |
| 314 | no-change | — | MaterialBaker concrete-vs-abstract; Material.withSuffix 26.3-only |
| 315 | no-change | — | EndCrystalRenderer submitModel null arg + beam-target shouldRender 26.3-only |
| 317 | no-change | — | CreateWorldScreen datapackWorldRegistries -> datapackWorldgen rename |
| 318 | no-change | — | DisplayRenderer culling is the item-92 tick-position form |
| 319 | no-change | — | ProjectileDeflection 3-arg + REVERSE -0.5 already Rust form; ShulkerBullet redirect 26.3-only |
| 320 | no-change | — | Enchantment holderSet/homogeneousList same JSON shapes; Builder Holder.direct 26.3-only |
| 322 | changed | 30 (est.) | PortalShape.FRAME is obsidian, not the 26.3-only #nether_portal_frame tag; portal_frame fixed to is_block(obsidian). Nether-portal harness repaired (state_properties: NBT `Properties` vs JSON `properties`) and passes 2626/2626 |
| 323 | no-change | — | TrialSpawnerConfig codec rename only; BlockEntity log arg order cosmetic |


Item 192 closed with 233 (2026-11-14): its powder-snow half was committed
in 14aacfc3 and its interpolation half is 233's rework.

Follow-up debt from item 233: 26.2's `hurtMarked` motion packet
(`SetEntityMotion` at the end of `sendChanges` when `hurtMarked` is set,
to tracking players AND self) has no Rust plumbing — needs a `hurt_marked`
flag from Mob through EntityState. The client's advancement tree layout
computes positions itself and does not use the packet x/y yet (26.2's
client positions widgets from DisplayInfo x/y).

Follow-up debt from item 305: other `nav_move_to_entity` call sites
passing reach 0 for goals that call the 2-arg `moveTo(Entity, speed)`
(fox, frog, goat, illager, panda, piglin, riders, spear_use, tempt)
need the same reach-1 check in their own items — the 2-arg form builds
the path with `createPath(target, 1)` in both versions.

Follow-up debt from item 115: the Rust density engine samples in float
(26.3's `DensitySampler` model), but 26.2's density functions compute in
double (`DensityFunction.compute`). The Beardifier's formulas match
exactly; only the arithmetic precision differs (bury: 26.2
`clampedMap(length, 0, 6, 1, 0)` in double vs 26.3 `1 - sqrt(sq)/6` in
float; beard: same shape, double vs float). A faithful 26.2 port needs
the density engine in double — a programme-level rework, not one item's
change.

Entity-fluid harness (2026-11-14, local run): the reworked
`test_entity_fluid_parity.py` passes bit-exact — 560,630 lines,
byte-identical against both 26.2 jars (server-game + client). The
driver's release `world_probe` build exceeds its own 900 s timeout on
this VM (1.5 shared CPUs, LTO thin + codegen-units=1); the run was done
in pieces — the orphaned rustc finished the binary, then the fixture
and both oracle sides were run by hand and compared with
`test_rust_parity.compare`. Pre-build the release probe before running
a parity driver here, or the driver times out at the build step. The
oracle needed one 26.2-specific fix beyond the tracker reflection:
26.2's `getTrackerFor` calls `Fluid.is(tag)`, which reads the
`builtInRegistryHolder` field directly, so the oracle now sets that
field on its Unsafe-allocated Fluids (26.3's check goes through the
holder, which the method override already covered).

## Placement harness known gaps (2026-11-14, local run)

`tools/test_placement_parity.py` against the 26.2 jar: 90/18,780 fixtures
mismatch (down from 882 after item 127 cleared the noise group). The
remaining group is pre-existing and out of item 242's scope:

- noise_based_count (792): CLEARED by item 127 (2026-11-15) — the f64
  synth port + biome noises to double. Fixture-by-fixture comparison:
  all noise fixtures match.
- count_on_every_layer (90): identical positions and queries, different
  interleave order — vanilla 26.2's builder-based `getPositions` (full column
  scan before any position is consumed) vs the Rust stack-loop placer emitting
  a position as soon as its column scan completes. Needs a pull-order
  investigation, not a decoder fix.

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

## CI profile — port-verify v6.6, run 43575017292 (2026-11-14)

Dispatched on behaviour/26.2@bc42387a (rank 2 at 258/410). Completed
2026-11-14T10:21:07Z, conclusion success (the harness leg is
continue-on-error, so the conclusion is not the profile).

**4 pass / 0 oracle / 13 fail** — food, lan, light, resource_pack
pass; the other 13 fail. Unchanged from run 43535967019. D1 pass; D2
now FAILS (it passed on 43535967019): one protocol test fails without
an accepted gap —
`adapter::translate::v1_21_1::recipes::tests::vanilla_recipe_serializers_become_pinned_displays_with_canonical_item_ids`
(properties count 7, expected 9). Verified pre-existing on the
stashed 14aacfc3 base (2026-11-14), so the regression landed in one of
the batches between 218/410 and 258/410 — needs a bisect, not yet
done. (Locally the same run shows 441 pass / 3 fail: the recipe test
plus the two sandbox-network tests, which also fail on the base.) D3
fail (rank 2: 152/410 neither closed nor accepted at this commit);
D4–D6 blocked as before. Artifacts kept at
`~/workspace/mcir-ci/26.2-258/`.

**D2 regression fixed 2026-11-14 (item 129 batch).** Bisected by git log
on the file: the only horizon-protocol commit in the 218–258 range was
`f7e99ff1` (item 177, brewing), which removed `brewing_input` /
`brewing_reagent` from the fixed names in `Recipes::properties()` with
the comment "1.21.1 has no brewing property sets (added in 26.3)". That
confused the source with the target: the adapter outputs the 26.3
pinned form, and 26.3's `RecipePropertySet` registry defines nine
vanilla keys — verified by decompiling the class from both jars (26.3
has `BREWING_INPUTS` / `BREWING_REAGENTS`, 26.2 has only the seven).
A 1.21.1 server contributes no brewing recipes, so both sets are empty,
but they must still be written because the 26.3 client reads all nine.
Restored the two names and corrected the comment. Local
`cargo test -p horizon-protocol --lib` is now **442 pass / 2 fail**
(the two remaining failures are the pre-existing sandbox-network pair);
the recipe test passes. status.json `protocol_tests` updated.

## CI profile — port-verify v6.6, run 43609469257 (2026-11-14)

Dispatched 2026-11-14T15:18:31Z on `1a8bd427` (285/410 tree). Completed
2026-11-14T17:28:06Z, conclusion success (the harness leg is
continue-on-error, so the conclusion is not the profile).

**5 pass / 0 oracle / 12 fail** — first_person, food, lan, light,
resource_pack pass; the other 12 fail. First_person is new since run
43575017292 (the item-8 first-person oracle rework landed in this
tree). D1 pass (all 6 generators clean, tables reproduce). D2 FAILS
but only on the warning count: protocol tests are **444 pass / 0
fail** in CI (the recipe test fixed in the item-129 batch passes; the
two local sandbox-network failures pass on CI, which has network) —
the gate counts 6 warnings against a 0/0 bar, and all six are the two
pre-existing warnings counted across targets (the
`unused_doc_comments` at `horizon-protocol/src/packets/play.rs:2238`
and the `unused_mut` at `game/creatures.rs:1361`, each in lib +
lib-test). D3 fail (rank 2: 125/410 neither closed nor accepted at
this commit); D4 blocked (profile stamped for `1a8bd427`, HEAD is
`28def2627f44` — re-record the sweep for this tree); D5 blocked (no
harness-subset.txt); D6 blocked (no playtest.md). Artifacts kept at
`~/workspace/mcir-ci/26.2-285/`.

## Item 8 closed (2026-11-14) — remote.rs / first person

Rank 2 now 284/410. The item's real changes, all verified against the
26.2 jars by decompilation:

- **Swing packet carries the hand**: 26.2's `ServerboundSwingPacket`
  has a hand VarInt; 26.3 replaced it with the handless
  `ServerboundPunchPacket`. The Rust sent an empty swing, so a 26.2
  server could not read the hand. Now `Swing { hand: i32 => VarInt }`,
  sent on every `LocalPlayer.swing` (unconditional, as vanilla does).
- **Login takes the previous game type**: `setLocalMode(gameType,
  previousGameType)` (2-arg form) — the Rust kept the default as the
  previous mode instead of the server's.
- **Wall nudge**: 26.2 `aiStep` runs `moveTowardsClosestSpace` under
  `!noPhysics` alone; 26.3 added `!isPassenger()`. The passenger
  exclusion is removed.
- **F4 shader toggle**: `key.toggleSpectatorShaderEffects` was registered
  but never consumed. `GameRenderer.togglePostEffect` (26.3 renamed it
  `toggleSpectatorPostEffect`) flips `effectActive`; the Rust now has
  the flag, set on `setPostEffect`, cleared on `clearPostEffect`.
- **Pause keeps the dig**: 26.2 `pauseGame` does not call
  `stopDestroyBlock` (26.3 added it); F3+Escape no longer drops `mining`.
- **Creative entity attack sets no destroy delay**: 26.3 added
  `destroyDelay = 5` to `MultiPlayerGameMode.attack`; 26.2 sets the delay
  only in `startDestroyBlock` / `continueDestroyBlock` (creative, and
  on break completion).
- **Mining fatigue table**: 26.2 `Player.getDestroySpeed` uses the
  hardcoded 0.3 / 0.09 / 0.0027 / 0.00081 table, not 26.3's
  `pow(0.3, amplifier + 1)` (fatigue III is 0.0027, not 0.027).
  Fixed in `Effects::mining_speed_factor` and the first-person probe,
  with a test pinning the table.
- **First-person oracle reworked to 26.2**: `ItemInHandRenderer` under
  `GameRenderer.itemInHandRenderer` instead of the 26.3-only
  `FirstPersonHandsAndItems`; `LocalPlayer` 8-arg constructor; the
  singular `addBreakingBlockEffect(pos, direction)` (26.3's plural
  `addBreakingBlockEffects` with the sound flag does not exist in 26.2);
  a `SoundManager` shell swallowing the destroy hit sound every fourth
  tick. Harness: **4,205 first-person cases match the pinned client
  (382,720 output lines)**.
- Verified no-change: explosion knockback is `addDeltaMovement` (the
  adapter already appends `play_sound=true` for 26.2); particle
  randomization is 26.2's single-max-speed gaussian; chat signatures
  pass directly; `getViewYRot` passenger interpolation; underwater loop
  fade math (+1/−2, 40 cap) identical in both versions.

## Item 12 closed (2026-11-14) — lan_actions.rs / LivingEntity, Entity, Player

- **PlayerAction ordinals are 26.3-shifted** (wire bug, both directions):
  26.3 inserted `CHANGE_DESTROY_DIRECTION` at ordinal 1; 26.2 is 0 START,
  1 ABORT, 2 STOP, 3 DROP_ALL, 4 DROP_ITEM, 5 RELEASE_USE, 6 SWAP, 7 STAB
  (verified with `javap` on both client jars). Fixed the client sends
  (`remote.rs`), the host dispatch (`lan_actions.rs`) and the packet doc;
  removed the face-change send (26.2's `continueDestroyBlock` sends no
  direction-change packet). New ignored regression test
  `a_26_2_guest_abort_and_stop_use_26_2_ordinals` passes.
- **Glide start bars water only in 26.2**: `tryToStartFallFlying` is
  `!isFallFlying() && canGlide() && !isInWater()`; removed the lava arm
  from `lan_start_gliding` (26.3's `!isInLiquid()`).
- **Visibility**: `worn_visibility` gains `minecraft:piglin_brute` (26.2
  fools brutes with a piglin head); removed the 26.3 0..10 clamp from
  `visibility_percent`.
- **Restitution**: the bounce gate is strict `<` in 26.2
  (`movement_restitution.rs`).
- **Collide**: the entity query is `aabb.expandTowards(movement)` only;
  26.3 adds the maxUpStep expansion (`geometry/movement.rs`).
- **hurtServer knockback**: 26.2 knocks back whenever the source is not
  `NO_KNOCKBACK`, even when fully blocked (26.3 added the
  `fullyBlocked` skip). Dropped the gate in `living_hurt.rs` and flipped
  the health test to assert knockback on a fully-blocked hit (14/14 pass).
- **forceSendPlayerSupportBlocks is 26.3-only**: removed the function, its
  call and `Guest.support_blocks_sent`. 26.2 never resends support blocks.
- **Duplicate-position kick is 26.3-only** (`receivedPositionThisTick`):
  removed the kick; `position_this_tick` stays for `known_movement`
  bookkeeping.
- Verified no-change: swing-back echo, `handleMovePlayer` inline impulse
  reset, suggestions inline parse with the 1000 cap, sign-update bool,
  `flushDirty` unpositioned holders, BedRule / `try_sleep`, unconditional
  blocked knockback, `isPushable` without the ticking gate,
  `teams_allow_damage` inline form, no 2019/2020 dig events, server-side
  `aiStep` interpolation dead, `canSimulateMovement`, `handlePortal`,
  single-boolean `Invulnerable`, water-only ridden float,
  `handleMoveVehicle` direct fields, creative-slot drop, `noBlocksAround`,
  `createWitherRose` flags 3, `drop` randomly=false with unconditional
  swing, `startUsingItem` (no 26.3 equipment update),
  `onEquippedItemBroken` (effects stop via per-tick
  `collectEquipmentChanges`), Team join on load, `teleport_rider`.

## Item 86 closed (2026-11-14) — game.rs / Farmland, TurtleEgg, Bucket, GameRenderer

- **Farmland trample used 26.3's spawn-protection gate**: 26.2's
  `FarmlandBlock.fallOn` tramples for every player (mobGriefing only
  gates mobs). Fixed `Rule::Farmland` to
  `allowed = entity == Player || mob_griefing()`.
- **Turtle-egg destroy used the same 26.3 gate**: 26.2's
  `canDestroyEgg(level, entity)` lets every player destroy (Turtle/Bat
  never, non-living never, other living only with mobGriefing). Fixed
  `destroy_egg` the same way.
- **Bucket evaporate smoke was the 26.3 form**: 26.2 calls
  `level.addParticle` eight times at the block corner plus a uniform
  `nextFloat()` on each axis, and `Level.addParticle` is a no-op on a
  server (no `ServerLevel` override in the 26.2 jar), so nothing is
  sent. Fixed `evaporation_smoke` to draw 8 uniform in-cube puffs on the
  acting client only (the serving game returns early: a guest's turn and
  a dispenser draw nothing) and removed the LAN `sendParticles` packet.
  The `spread()` helper + test (26.2's `handleParticleEvent` gaussian
  spread of a received packet) is a different mechanism and stays.
- **Spectator block outline**: the Rust never showed one; 26.2's
  `shouldRenderBlockOutline` shows it over a block with a menu provider
  (`getMenuProvider != null`). `can_build` now includes spectators and
  the pick filter is mode-aware (spectator -> `block_opens_menu`).
- Citation fixes: the spinning-effect state lives in `GameRenderer`
  (26.3 moved it to `LocalPlayer`); the OIT gate is
  `GameRenderState.useShaderTransparency()` (26.3 renamed it
  `useImprovedTransparency`); the post-effect list is what
  `GameRenderer.render` applies after `doEntityOutline` (26.2's `update`
  is camera-only; no `LocalPlayer.postEffects` list).
- Verified no-change: bobHurt/bobView (rename only), checkEntityPostEffect
  (`setPostEffect`/`clearPostEffect`), render/renderLevel/tick/update/
  resetData (spinning in `GameRenderer.tick`, post-effect inline after
  `doEntityOutline`), `preparePostEffects`/`render3dHud`/
  `spectatedEntityPostEffect` (26.3-only), `Block.fallOn` (no
  `getFallDistanceReduction` in 26.2; all 1,196 blocks export 0.0; Hay
  0.2 / Bed 0.5 / Honey 0.2+event 54 / Slime 0.0 / PowderSnow sound-only /
  DripstoneTip distance+2.5 x2.0 match the 26.2 overrides), `Level.setBlock`
  4-arg bodies identical (the mdiff 48-line diff was an overload
  mismatch), `Level.destroyBlock` (2001 unless BaseFireBlock),
  `playerDestroy` (signature only), `playerWillDestroy` (plain 2001),
  `spawnDestroyParticles` (2001), `withPropertiesOf` (hasProperty guard),
  `dropFromBlockInteractLootTable` (visibility), `getEntities`/`playSound`
  identical, `EnderDragonFight.setDragonKilled` (same podium; no leaves
  in the End), `Gui.addInitialScreens` (platform rename),
  `openChatAndAddText` (sole caller always passes COMMAND),
  `FallingBlockRenderer.shouldRender` (state != cell), `CakeBlock.eat`
  (bites<6 setBlock flags 3; food 2, saturation 2*0.1*2=0.4),
  `LilyPadBlock.entityInside`, `RconConsoleSource` (level 4, named Rcon,
  spawn lower corner), `BucketItem.emptyContents` placement logic.
- Known gap (not fixed): `Gui.tick` wraps `screen.tick()` in a try/catch
  -> CrashReport "Ticking screen"; the Rust has no equivalent screen-tick
  crash wrapper (panics propagate; no Minecraft-style crash report).
- Checks: `cargo check -p horizon-client -p horizon-world
  -p horizon-client-render --all-targets` clean (only the pre-existing
  `unused_mut` at `game/creatures.rs:1361`); `lan_particles` tests 5/5.
  Block-landing parity harness vs the 26.2 oracle: block-landing parity harness vs the 26.2 oracle: 5,597,105 registered lines match (client + server)

## Items 147 + 374 closed (2026-11-14) — game/cushions.rs, cushion.rs / BlockAttachedEntity, Cushion

- **Cushion is 26.3-only**: 0 cushion entries in either 26.2 jar
  (server-game.jar and client.jar). `Cushion`, `CushionItem` and
  `CushionRenderer` removed in 26.2; `BlockAttachedEntity` exists in
  26.2 (144 lines decompiled) as the base for paintings, item frames
  and leash knots; `Vec3.atCenterOfWithY` is 26.3-only.
- **Removal (item 374)**: the Cushion entity/item/renderer, its
  `dye_index`, `Target::Cushion`, the `minecraft:cushion` load/save
  paths, LAN cushion states, cushion seats/rides, cushion blast and
  thunder handling, and the `cushion_placement` probe are gone.
  Generic logic that lived in `cushions.rs` moved, not deleted:
  `dismount_in_portal` / `dismount_removed_vehicle` to `passengers.rs`
  (portal dismount applies to any vehicle) and `state_suffocating` /
  `state_view_blocking` (26.2's two-arg `BlockState.isViewBlocking`)
  to `render_tick.rs`.
- **Real fix (item 147)**: 26.2's `BlockAttachedEntity.kill` is
  `Entity.kill(level)` — the death is the entity's own event,
  unattributed. 26.3's `kill(level, entity)` attributes it to the
  killer (and adds `onKilled` / `tickAtCheckInterval`). Painting and
  frame `ENTITY_DIE` are now unattributed on player breaks and on
  blast breaks; `explosion_kill_credit()` is removed (its only uses
  were those two sites) along with `crystal_blast_player()` (its only
  caller was the kill credit; `blast_by` itself stays — it is read at
  `dragon.rs:3780` for chain-blasting crystals). The tick gate was
  already the 26.2 `checkInterval++ == 100` form (fires tick 101).
- **Frame `BLOCK_CHANGE` keeps its causedBy**: 26.2's
  `ItemFrame.dropItem(level, causedBy)` posts `BLOCK_CHANGE` attributed
  to `causedBy`, and `BlockAttachedEntity.hurtServer` passes the
  damage source's entity — so an explosion-broken frame's
  `BLOCK_CHANGE` is attributed to the blast's direct source entity
  (the exploding mob), via the new `explosion_direct_source()`; a
  primed-TNT or end-crystal blast has no `Who` and stays
  unattributed. (Paintings post no `BLOCK_CHANGE`:
  `Painting.dropItem` is sound + item only.)
- **Two tests pinned 26.3 behaviour and are updated to the 26.2
  truth**: `entity_die_explosions` expected a fifth event for the
  dropped item (26.2's `ItemEntity.hurtServer` just discards — no
  `ENTITY_DIE`, in both versions) and mob attribution for
  stand/painting/frame; it now asserts 4 events, all self-sourced, in
  both the caused and uncaused blast iterations. The stand test
  `a_hit_from_damage_posts_entity_damage_then_entity_die_for_its_causing_entity`
  pinned the same 26.3 attribution and was already failing on the
  branch (its code, `armor_stands.rs`, was fixed to 26.2 in the
  earlier stand batch; only the expectation lagged) — renamed to
  `..._entity_damage_attributed_then_entity_die_unattributed` and now
  asserts an attributed `ENTITY_DAMAGE` then an unattributed
  `ENTITY_DIE`.
- **Cushion-placement harness**: `test_cushion_placement_parity.py`
  is not applicable to 26.2 — its oracle imports the 26.3-only
  `blockscan` package and will not compile against the 26.2 jar, and
  the Rust probe was removed with the feature. The driver and oracle
  stay in place for the 26.3 tree (parity harnesses are never
  deleted); the harness is not in CI's 17-harness profile.
- Checks: `cargo check -p horizon-client -p horizon-world
  --all-targets` clean (only the two pre-existing warnings);
  `entity_die_tests` 2/2, `stand_command_hurt_tests` 12/12,
  `kill_die_vibration_tests` 1/1, `vibration_tests` 6/6,
  `lan_particles` 5/5.

## Item 109 closed (2026-11-14) — game/game_tests.rs / GameTest framework

- **26.2 has no per-dimension GameTest model**: `TestData` has no
  dimension field (26.3 added it with codec default Overworld);
  `GameTestRunner` holds a `ServerLevel` (26.3: `MinecraftServer`);
  `GameTestBatchFactory` groups by environment only (`BatchKey` is
  26.3-only); `StructureGridSpawner` is a single flat grid (26.3:
  per-dimension `DimensionGridState` grids).
- **Rework**: `TestDef.dimension` removed (parse, describe row,
  sample); `Info.dim` removed (`new`, `copy_reset`,
  `verify_batches`, `test_info_at`, `test_info_for`, `run_test_at`);
  the `Runner` gains its `level` (set at `track_and_start` from the
  level in place) and `last_batch` is now `Vec<Pos>`;
  `Grid.states` / `DimGrid` collapsed to a single
  `Option<GridState>`; `run_batch`'s `onBatchStart` resets the corner
  and the row bounds but NOT the row count (26.2 does not reset
  `currentRowCount`); `tick_game_tests` ticks only while the runner's
  level is in place; `spawn_test` no longer switches levels
  (`in_test_level` removed); `has_tests_in` is the runner's level
  with tests pending.
- **Say is level-scoped**: `ReportGameListener.say` reaches only the
  players in the test's level (26.3 tells the whole server). The
  flush now uses `hub.broadcast_level`
  (`PlayerList.broadcastAll(packet, dimension)`) and defers `Players`
  lines until the runner's level is in place; with no runner left
  (a halt cleared it) they go to the level in place.
- **Summary coordinate links are 26.3-only**:
  `TestCommand.outputPlayerCoordinates` / `outputTestCoordinates`
  do not exist in 26.2 and `TestSummaryDisplayer` is a record with no
  dimensions set. `Out::Coordinates`, `summary_coordinates` and
  `output_test_summary_coordinates` are removed; the
  coordinate-feedback E2E test is reworked to a summary-only test.
  `TestInstanceBlockEntity.runTest` itself is identical in both
  versions.
- Checks: `cargo check -p horizon-client --all-targets` clean (only
  the two pre-existing warnings); `game_tests` unit tests 10/10;
  summary E2E (ignored) passes.
- **Pre-existing failure, NOT caused by this item** (verified by
  bisect on `c03452c8` — identical failure, same line and values):
  `added_issuer_actor_missing_guest_and_restoration_controls` fails
  at `test_summary_context_tests.rs:187` (`game.dim` Overworld vs
  Nether after `test_source_output` to a guest while in the Nether;
  no level-open error logged). Needs a follow-up in the
  `for_player` / guest-turn level-restore path, outside the GameTest
  framework.
- WIP-checkpoint rule applied: the rework was committed and pushed
  as `b9f704d9` once it compiled, before test verification.

## Item 106 closed (2026-11-15) — mobs/spawn_selection.rs / natural spawning

- 3 of 15,620 harness lines diverged (`tools/test_natural_spawn_parity.py`,
  artifacts in `.local/rust-natural-spawn-parity/`). Two root causes, both
  in the probe / shared spawn-selection code:
- **Quart-origin spawn costs.** The probe's `spawn_cost` read the biome at
  block coords, but vanilla's `SpawnState.canSpawn` goes through
  `getRoughBiome` — the chunk's *noise* biome at the quart cell's origin
  block (the oracle's `getNoiseBiome` samples `biomeAt((x>>2)<<2, ...)`).
  A test at a biome boundary can sit in a quart cell whose origin is
  across it: at `(2,43,3)` and `(3,30,9)` the origin block is crimson
  forest (no skeleton spawn cost, so `canSpawn` is true) while the block
  itself is soul sand valley (cost 0.7, energy over the 0.15 budget).
  Fixed the probe's `spawn_cost` to the quart-origin biome; the real
  `GameLevel` already used `zoomed_biome`. Verified numerically before
  the fix (block-coord energies 0.2534 / 0.3124 false vs quart-origin
  no-cost true, matching vanilla).
- **Fresh pufferfish dimensions.** A fresh pufferfish is puff state 0, so
  `Pufferfish.getScale(0)` = 0.5 makes it 0.35 wide, not the registry 0.7.
  The 0.7 box falsely overlapped the spider's spawn box at `(9,75,8)` and
  `checkSpawnObstruction` failed it. Added `fresh_dimensions` (pufferfish
  x0.5), used by `mob_type` in both impls, the probe's case-entity boxes,
  and `neutral_box`.
- **Wrong turn, reverted in full.** The obstruction predicate was first
  narrowed to `canBeCollidedWith`-only (boats / happy ghasts / shulkers),
  which regressed the harness to 387 differing lines: vanilla's
  `isUnobstructed` counts every non-spectator entity with `blocksBuilding`
  (true for all living entities — `LivingEntity` sets it), and the run's
  own spawns DO block repeats at the same spot (squid / sheep / enderman
  cases). `canBeCollidedWith` is `noCollision`'s rule, a different method.
  The doc comment on `spawn_obstruction` now records the distinction.
- Harness: all 15,620 natural spawn cases match vanilla (97,604 attempts,
  14,068 type checks passed, 10,850 spawns).
- WIP checkpoints `35afa6d1` and `d0b1b62d` pushed mid-batch.
- Environment notes: the fast-profile `client_probe` link failed once with
  rust-lld "undefined hidden symbol" from stale incremental artifacts —
  fixed by `cargo clean -p horizon-client-lib --profile fast`; the
  driver's 900 s build timeout was worked around by pre-building the fast
  probe first.

## Batch of 8 rank-2 A items closed (2026-11-15) — 131, 171, 214, 236, 239, 244, 246, 248

Seven no-change (verified against the jars, no code change) and one changed
(214, the RuleTest rework). Rank 2 290/410 -> 298/410.

## Item 131 no-change (2026-11-15) — mesher/fluid.rs / world_border_render

- `crates/horizon-client-render/src/mesher/fluid.rs`, 4 classes, 0 named.
  The RenderSystem diffs are the 26.3 SDL/RenderPearl vs 26.2 GLFW/GLX
  platform rework; the Rust's fluid mesher names none of the 67 changed
  methods (only a comment mentions RenderSystem, line 199).
- Follow-up: the `world_border_render` oracle
  (`VanillaWorldBorderRenderOracle.java`) does NOT compile against 26.2 —
  it uses 26.3-only API (`RenderPass`, `DynamicGpuData`, `GpuBuffer`,
  `WorldBorderRenderer.prepare(state, cam, distance, depthFar)`; 28
  compile errors). It needs a 26.3->26.2 oracle rework (same class as
  item 127's noise oracle). Not run on this branch.

## Item 171 no-change (2026-11-15) — world/level/biome

- 9 classes, 0 named. Sampling math is identical; the diffs are the API
  rework: `BiomeResolver.getNoiseBiome` takes the `Sampler` in 26.2 (no
  `@FunctionalInterface` / `createResolver` / `getBiomesWithin`);
  `Climate$Sampler` uses `DensityFunction` double `compute` in 26.2 vs
  `DensitySampler.Bound` float `sampleValue` in 26.3 (the known item-115
  precision debt); `ParameterList` uses `RTree.create(values)` in 26.2 vs
  the 19-children constructor in 26.3; `Parameter.distance` is public in
  26.2; `ParameterPoint.fitness` is private in 26.2 vs public in 26.3;
  codec renames `RegistryCodecs.holder` -> `RegistryFileCodec.create`,
  `codec()` return widened.
- Follow-ups: (1) 26.2-only gap — `Sampler.spawnTarget` /
  `findSpawnPosition` (called only by `MinecraftServer:480` for the
  dedicated-server spawn chunk search) is not implemented in the Rust;
  documented gap, not a sampling change. (2) None of the biome harnesses
  (biome_generation, biome_source, biome_source_codec, datapack_worldgen,
  flat_generation, material_stage, structure_layout, worldgen) has been
  run on the 26.2 branch — no harness artifacts exist in `.local/` and
  none are mentioned in PORT-NOTES.

## Item 236 no-change (2026-11-15) — core (Holder / Vec3i / Direction)

- 11 classes, 0 named. Diffs are renames / inlined helpers with identical
  semantics: `Holder.getRegisteredNameIfPresent` inlined into its one
  caller; `Vec3i.offsetCodec` strict `<` in 26.2 vs `<=` in 26.3 (already
  handled, item 242) and `differsHorizontally` is 26.3-only;
  `Direction` drops the `Directional` interface, `getStep` ->
  `getUnitVec3i` rename; `HolderOwner.canSerialize` ->
  `canSerializeIn` with the `HolderGetter` / `RegistryLookup` interface
  hierarchy reshuffled; `BlockPos$MutableBlockPos` constructs
  `new BlockPos(this)` instead of `(x, y, z)`. `Direction$Axis` (46
  lines): 26.2 enum constants carry `(name, ordinal, lowercase)` and
  `getPositive` / `getNegative` / `choose` are abstract per-constant
  methods vs 26.3's concrete switch methods — identical semantics.

## Item 239 no-change (2026-11-15) — gfx/core_sky.rs / translucency

- 2 classes, 0 named. `ColorTargetState` is NOT in either 26.3 jar at
  `com/mojang/blaze3d/pipeline/` — in 26.3 it moved to
  `com/mojang/renderpearl/api/pipeline/ColorTargetState`. The diff is the
  RenderPearl-vs-Blaze3D package move only; the Rust's sky rendering
  uses its own pipeline and names none of the changed methods.

## Item 244 no-change (2026-11-15) — gfx/entity_batch.rs / block_entity_render

- 1 class, 0 named. `RenderType` diff is the RenderPearl-vs-Blaze3D
  import move plus the outline-texture lookup (26.3 uses the named
  outline texture binding; 26.2 uses the first texture) and a removed
  `hasBlending` field. The Rust's entity batching does not use Mojang's
  `RenderType` (own render types); no behaviour the Rust depends on
  changes.

## Item 246 no-change (2026-11-15) — cosmetics/item.rs

- 1 class, 0 named. `Item$Properties` (64 lines): 26.3-only `villagerFood`
  and `potPattern` methods removed in 26.2; components `VillagerFood`,
  `BrewingFuel`, `CookingFuel`, `Compostable`, `MobVisibility`,
  `BlockTransformers` absent in 26.2. The shared methods
  (`usingConvertsTo`, `repairable`, `componentInitializer`) are
  identical. The Rust's cosmetics/item.rs does not reference the
  26.3-only methods.

## Item 248 no-change (2026-11-15) — util/context

- 2 classes, 0 named. `ContextMap$Builder` diff is a map-implementation
  swap (`Reference2ObjectOpenHashMap` in 26.3 vs `IdentityHashMap` in
  26.2) with renamed methods (`set`/`get` in 26.3 vs
  `withParameter`/`withOptionalParameter`/`getParameter` in 26.2);
  `ContextMap` itself is `final` with a static `EMPTY` and `builder()`
  factory in 26.3 vs a plain class in 26.2. Semantics identical; the
  Rust's context code does not call these methods.

## Item 214 changed (2026-11-15) — world/level/levelgen/structure/templatesystem / rule_test

- 3 classes, 0 named, but the diff is a real API rework with two genuine
  26.2 divergences in the Rust, both found by the reworked harness:
- **RuleTest is 2-arg in 26.2.** 26.3's `test(state, pos, random)` (with
  `height_match`, `not`, `all_of`, `any_of` rule types) vs 26.2's
  `test(state, random)` with only six registered types
  (`always_true`, `block_match`, `blockstate_match`, `tag_match`,
  `random_block_match`, `random_blockstate_match`). The Rust accepted all
  four 26.3-only types and carried a position parameter used only by the
  `Height` node. Removed the four decode arms and `Not`/`All`/`Any`/
  `Height` nodes, dropped the position parameter from `RuleTest::test`
  (callers in `block_feature.rs`, `ore_feature.rs`,
  `structure/processor.rs`, and the fixture updated), and simplified
  `pure_heights` (no Y ranges to collect in 26.2; the `Option` only
  reports whether the rule draws from random — the ore-feature cache
  stays correct with the empty ranges). 26.3-only definitions now
  decode-error on both sides.
- **26.2's `BlockState.CODEC` is object-only with NBT names.** 26.3's
  `CODEC` is an `Either<Block, BlockState>` accepting a bare block id or
  a full object with `id`/`properties`; 26.2's `CODEC` is the object form
  with `Name`/`Properties` only (26.3's `FULL_CODEC`). Probed directly
  against the 26.2 oracle: bare `"air"` and `{"id":...}` forms
  decode-error, `{"Name":...}` decodes ok; probability -1 is accepted
  in both. Fixed `BlockStateJsonCodec::decode(json, full=true)` to read
  `Name`/`Properties` only (the `full=false` compact form is unchanged,
  26.3-model), and `RuleTest`'s `block_state` field now decodes with
  `full=true`. `JigsawBlockInfo` has no Rust counterpart: 26.3's record
  is `(pos, state, jointType, @Nullable name, ...)` with a `parse(info)`
  factory; 26.2 holds the whole `StructureBlockInfo` with a non-null
  `name` and an `of(info)` factory — a rename only.
- Harness: `tools/test_rule_test_parity.py` reworked for 26.2 (oracle
  2-arg call; driver `rule_types` 10 -> 6 with the scope text updated;
  pinned state count 35723 -> 32366) and now passes: **1,251,780
  replacement-rule output lines match Rust on both the client and the
  server jars** (68,956 codec inputs, 1,660,472 requested tests). The
  first two runs each found one real divergence (above), so the harness
  earned its keep.
- WIP checkpoints `d69ec30a`, `092b0d3d`, `3d1f8ef8` pushed mid-batch.
- Follow-up debt (same class, other harnesses own them): `decode(...,
  false)` — the 26.3 compact block-state form — is still used for vanilla
  `BlockState.CODEC` fields at `state_provider.rs:183,393,397` (simple /
  weighted / noise providers), `generator.rs:1086`
  (`NoiseGeneratorSettings` default_block/default_fluid — its comment
  claims the id-alone form is accepted in 26.2, which the probe above
  disproves for `BlockState.CODEC`), and
  `material_condition/library.rs:195` (`result_state`). Each needs its
  own harness to confirm whether the field's vanilla codec is the
  object-only 26.2 form. The `block_state_codec` harness itself needs a
  26.2 rework: its oracle references `FULL_CODEC`, which does not exist
  in 26.2, and its corpus uses the `id`/`properties` object form.

## Batch of 8 rank-2 A items closed (2026-11-15) — 249, 250, 251, 255, 256, 259, 275, 277

Six no-change, two changed (250, 277). Rank 2 298/410 -> 306/410.

## Item 249 no-change (2026-11-15) — world/item/crafting/display

- 3 classes, 0 named. `SlotDisplay$TagSlotDisplay`: 26.3 holds a resolved
  `HolderSet<Item>`; 26.2 holds a `TagKey<Item>` (resolved lazily via the
  registry). The Rust's `Ingredient` expands tags at load time into
  `items` + `order` (HolderSet order via `item_tag_order`) — the eager
  26.3 shape, not 26.2's lazy TagKey. Observable difference only on
  datapack reload (a tag's contents changing after load); the
  ghost_recipe harness (unrun on 26.2) owns that verification.
  `SlotDisplayContext`: 26.3's context has `REGISTRIES` (RegistryAccess)
  only; 26.2 adds optional `FUEL_VALUES` and `REGISTRIES` is a
  `HolderLookup.Provider`. The Rust's recipe context does not pass fuel
  values. No behaviour the Rust depends on changes.

## Item 250 changed (2026-11-15) — world/level/levelgen/feature/stateproviders

- 1 class, 0 named, but the diff is a real codec rework with a genuine
  26.2 divergence in the Rust (the follow-up from item 214): 26.3's
  `BlockStateProvider` is an interface with `TYPED_CODEC`,
  `STATE_OR_PROVIDER_CODEC` (xor of `BlockState.FULL_CODEC` +
  TYPED_CODEC), `DIRECT_CODEC` and a holder `CODEC`; 26.2's is an
  abstract class with a single byNameCodec-dispatch `CODEC` and static
  `simple(state)` / `simple(block)` factories (26.3: `of` /
  `holderOf`). Every provider's state field uses 26.2's object-only
  `BlockState.CODEC` (verified in the jars: `SimpleStateProvider`
  `BlockState.CODEC.fieldOf("state")`; `WeightedStateProvider`
  `WeightedList.nonEmptyCodec(BlockState.CODEC)`; `NoiseThresholdProvider`
  `BlockState.CODEC` for `default_state` / `low_states` /
  `high_states`; `NoiseProvider` `BlockState.CODEC.listOf()` for
  `states`; `WouldSurvivePredicate`'s `state` field). The Rust decoded
  all five of those fields with the compact form (`decode(..., false)`,
  accepting bare strings and `id`/`properties` objects). Fixed to the
  object-only form (`true`): `state_provider.rs` simple `state`,
  weighted entries, and `predicate_state` (would_survive); `noise.rs`
  `default_state` and the `states` lists. The direct-state attempt in
  `StateProvider::decode` already used `true`.
- Follow-up: the `state_provider` harness cannot run on 26.2 yet — its
  oracle compilation fails with 97 errors across three helpers
  (`VanillaMaterialConditionOracle` 76, `VanillaStateProviderOracle` 11,
  `VanillaAnchorOracle` 10), all 26.3-only APIs (packages
  `densityfunction`, `material/condition`, `material/rule` do not exist
  in 26.2; `RegistryInfoLookup` shape differs). Needs a full 26.3->26.2
  oracle rework, same class as items 127/131.
- WIP checkpoint `37c9f5b1` pushed mid-batch.

## Item 251 no-change (2026-11-15) — client/renderer/weather

- 1 class, 0 named. `WeatherEffectRenderer` (123 diff lines) is the
  26.3 RenderPearl/OIT render-pass rework (`prepare` -> `render`,
  `setUniform` -> `bindTexture`, `OitStage` -> `OutputTarget.WEATHER_TARGET`,
  pipeline selection via `useShaderTransparency`). The Rust's weather.rs
  is its own wgpu renderer and names none of Mojang's render types; the
  `weather_render` harness (unrun on 26.2) owns the rendering
  verification.

## Item 255 no-change (2026-11-15) — world/level/levelgen/feature

- 1 class, 0 named. 26.3's `SimpleBlockFeature` is a record
  `(Holder<BlockStateProvider> toPlace, boolean scheduleTick)` with a
  MapCodec; 26.2's is the old `Feature<SimpleBlockConfiguration>` class
  with `place(FeaturePlaceContext)`. The Rust's `SimpleBlockFeature`
  already decodes `to_place` as a direct provider (not a holder) with an
  optional `schedule_tick` defaulting false, and its place logic matches
  26.2's (optional state, `canSurvive`, double-plant / mossy-carpet
  special cases via placement ops, schedule tick). The `simple_block`
  harness (unrun on 26.2) owns the verification.

## Item 256 no-change (2026-11-15) — world/level/levelgen/structure

- 5 classes, 0 named. `StructureCheck`: 26.3 adds a
  `chunksWithoutStartsInStorage` LongSet cache (131072 cap) and a
  `climateSampler` field; 26.2 passes the pos key straight to
  `tryLoadFromStorage` — the cache is a pure optimization with
  identical results. `StructurePiece.addChildren` takes a
  `StructurePiecesBuilder` (26.3) vs a `StructurePieceAccessor` (26.2);
  `BoundingBox.getCenter` / `intersects` are the same values,
  restructured. No behaviour the Rust depends on changes.

## Item 259 no-change (2026-11-15) — server/commands/data

- 3 classes, 0 named. `TagCommand`: 26.3 uses the
  `CommandResponseTracker.messages` helpers; 26.2 uses `Sets` + inline
  messages — the command behaviour (add/remove/list tags, messages) is
  identical, only the response plumbing is restructured.
  `BlockDataAccessor` / `StorageDataAccessor`: `ArgProvider` (26.3) vs
  `DataProvider` (26.2) Brigadier-provider rework; the accessor
  behaviour (get/set NBT on a block entity / storage) is unchanged.
  The Rust's `game/commands/data.rs` implements `/data` and `/tag` with
  its own argument parsing.

## Item 275 no-change (2026-11-15) — level/query.rs / block/state

- 2 classes, 0 named. `BlockStateBase.calculateSolid` logic is identical
  in both versions (forceSolidOn -> true, forceSolidOff -> false, null
  cache -> false, empty shape -> false, bounds size >= 0.729... -> true,
  else Y size >= 1.0); 26.3 only wraps every return in
  `SolidDebugger.logAndGet` (debug logging). `blocksMotion()` (not
  cobweb / bamboo_sapling and `isSolid()`) is present in 26.2 as
  established. `shouldRedstoneWireConnectTo` is 26.3-only; the Rust
  never calls it. `isViewBlocking`: 26.3's `StateArgumentPredicate<AABB>`
  vs 26.2's plain `StatePredicate` — the Rust already implements the
  26.2 two-argument form in `render_tick.rs`. `DiodeBlock`: 26.3-only
  abstract `codec()` + the `RedstoneWireBlock` rename (26.2:
  `RedStoneWireBlock`); the power-read logic is identical.

## Item 277 changed (2026-11-15) — world/level/block/state

- 3 classes, 0 named. `BlockState` / `StateHolder` diffs are the codec
  story from item 214 (26.3 `FULL_CODEC` + Either vs 26.2 object-only;
  `id`/`properties` vs `Name`/`Properties`), already fixed and verified
  by the rule_test harness. `Properties`: 26.3's default
  `isSuffocating` is `state.is(#causes_suffocation) &&
  isCollisionShapeFullBlock`; 26.2's is `state.blocksMotion() &&
  isCollisionShapeFullBlock`. The `#causes_suffocation` tag file exists
  only in the 26.3 jar (52 bytes) — it is absent from 26.2 — so the
  Rust's `state_suffocating` default arm (`full_collision &&
  block_in_tag("minecraft:causes_suffocation", ...)`) was always false
  under 26.2 data: no suffocation overlay ever showed for solid blocks.
  Fixed to the 26.2 form using the already-populated `legacy_solid`
  fact (from `block-behaviours.json`, also used by the fluid mesher)
  with the cobweb / bamboo-sapling exclusion — the same `blocksMotion`
  rule horizon-world already uses. The explicit arms (dirt_path /
  farmland / mud / soul_sand, pistons, leaves / glass / copper_grate,
  shulker boxes) are unchanged. `PushReaction` default NORMAL (26.2)
  vs PUSH_PULL (26.3) was already known; `fallDistanceReduction` is a
  26.3-only Properties field (26.2 blocks all export 0.0, item 86).
- WIP checkpoint `37c9f5b1` pushed mid-batch.

## Batch of 10 rank-2 A items closed (2026-11-15) — 297, 298, 299, 300, 301, 303, 304, 308, 309, 310

All ten no-change (verified against the jars, no code change). Rank 2
306/410 -> 316/410. Rate sample: 88 -> 98 items.

## Item 297 no-change (2026-11-15) — world/level/storage/loot/functions

- 2 classes, 0 named. `SetNameFunction`: 26.3's single
  `Optional<Holder<LootItemCondition>> condition` vs 26.2's
  `List<LootItemCondition> predicates` — the loot list form already
  ported in item 101; `context.getOptional` -> `getOptionalParameter`
  is the ContextMap rename (item 248). `ExplorationMapFunction$Builder`:
  26.3 takes a `HolderSet<Structure>` destination in the constructor;
  26.2 holds a `TagKey<Structure>` with a `setDestination` setter and a
  `DEFAULT_DESTINATION`. The Rust's `ExplorationMap::from_json` already
  implements the 26.2 form (tag string, default
  `minecraft:on_treasure_maps`, documented in the code).

## Item 298 no-change (2026-11-15) — item/hash/raw.rs

- 2 classes, 0 named. `TrimMaterial`: 26.3's record is `(Identifier
  paletteId, ...)`; 26.2's is `(MaterialAssetGroup assets, ...)` — the
  trim palette rework already ported in item 61 (asset_name +
  override_armor_assets, trims/color_palettes/trim_palette).
  `BannerPattern`: `RegistryCodecs.holder` -> `RegistryFileCodec.create`
  rename only.

## Item 299 no-change (2026-11-15) — client/gui/font

- 2 classes, 0 named. `AtlasGlyphProvider$Instance` diff is the
  RenderPearl -> Blaze3D import move only; `$1` (synthetic) has no diff
  at all.

## Item 300 no-change (2026-11-15) — client/resources/model/cuboid

- 2 classes, 0 named. `CuboidModelElement`: 26.3's record field is
  `@Nullable Direction shadeDirectionOverride`; 26.2's is `boolean shade`
  (default true, with a type check throwing on a non-boolean). The Rust's
  `Element::decode` already reads both (`shade` bool defaulting true,
  plus `shade_direction_override`), with the 26.2/26.3 split documented
  in the field comment.

## Item 301 no-change (2026-11-15) — network/chat

- 2 classes, 0 named. `MessageSignature` / `Packed`: 26.3 uses Netty
  `ByteBuf` + `VarInt` StreamCodec; 26.2 uses `FriendlyByteBuf` — the
  buffer API only, same wire format (Packed id + 1, 256-byte signature,
  as the Rust's `play.rs` documents and its test pins).

## Item 303 no-change (2026-11-15) — world/level/storage/loot

- 2 classes, 0 named. `LootTable$Builder`: 26.3 `apply(Holder<...>)`
  vs 26.2 `apply(LootItemFunction.Builder)` + `build()` — the loot
  list form already ported in item 101. `LootContext`: `getOptional` ->
  `getParameter` (getOrThrow) / `getOptionalParameter` is the ContextMap
  rename (item 248); the Rust's value-based context lookups have the
  same outcomes.

## Item 304 no-change (2026-11-15) — gfx/pack_atlas.rs

- 2 classes, 0 named. `ChunkSectionLayer`: RenderPearl -> Blaze3D
  imports; 26.3's enum constants carry an extra MULTIDRAW pipeline
  variant (removed in 26.2). `SectionCompiler` diff is 2 lines. The
  Rust's section mesh uses its own layers.

## Item 308 no-change (2026-11-15) — lan/conn.rs

- 4 classes, 0 named. `ServerConnectionListener`: 26.3 holds a
  `pendingConnections` ConcurrentLinkedQueue; 26.2 accepts channels
  directly via `acceptChannel`. The Rust's `lan/conn.rs` is its own
  Netty-free listener; connection-acceptance order is the same. The
  login-listener classes differ by 2-3 lines (inner-class plumbing).

## Item 309 no-change (2026-11-15) — horizon-items/blocks.rs

- 2 classes, 0 named. `WallHangingSignBlock` / `WallSignBlock`: 26.2
  carries a `CODEC` with a `wood_type` field; 26.3 removed the per-block
  codec. The Rust does not use Mojang's block codecs for signs (block
  types come from the registry data); sign behaviour is unchanged.

## Item 310 no-change (2026-11-15) — client/gui/components

- 7 classes, 0 named. `AbstractWidget.isValidClickButton`: 26.3 checks
  `button() == 1`, 26.2 `button() == 0` — the same left click under
  different numbering (SDL is 1-based, GLFW/winit 0-based). The Rust's
  UI activates on left click. `ScrollableLayout.getScrollAmount` is
  26.3-only; the rest of the diffs are 0-2 lines (AbstractButton,
  AbstractScrollArea, BossHealthOverlay, CycleButton, ImageButton have
  no diff at all).

## Batch of 10 rank-2 A items closed (2026-11-15) — 312, 313, 314, 315, 317, 318, 319, 320, 322, 323

Nine no-change, one changed (322). Rank 2 316/410 -> 326/410. Rate
sample: 98 -> 100 items (complete — see the tally note below).

## Item 312 no-change (2026-11-15) — world/level

- 3 classes, 0 named. `GameType`: 26.3 has an `OPTIONAL_STREAM_CODEC`
  (ByteBuf); 26.2 has `getNullableId` with a -1 sentinel. The Rust's
  26.2 adapter already uses the -1 sentinel (`layout_26_2.rs` writes
  `previous_game_type.map_or(-1, ...)`, `translate.rs` reads
  `(previous >= 0).then_some(...)`) — the item-8 login fix.
  `BaseSpawner.setEntityData(TypedEntityData, ...)` is 26.3-only; 26.2's
  spawner has no such method. `NaturalSpawner$SpawnPredicate.test`
  takes a `ServerLevel` in 26.3, not in 26.2 — the Rust's spawn
  selection (item 106, harness-passing) already uses the 26.2 form.

## Item 313 no-change (2026-11-15) — protocol/item/hash.rs

- 2 classes, 0 named. `RegistryOps` / `HolderLookupAdapter`: the
  `owner()` / `getter()` / `elementsLifecycle()` HolderOwner rework.
  The Rust does not port the class (only a doc comment in `hash.rs`
  mentions it); its registry ops are its own. No Rust surface.

## Item 314 no-change (2026-11-15) — client/resources/model/sprite

- 2 classes, 0 named. `MaterialBaker`: 26.3 is a concrete class with
  block/item atlas Preparations; 26.2 is abstract with just a missing
  sprite. `Material.withSuffix` is 26.3-only. The Rust's sprite baking
  is its own implementation.

## Item 315 no-change (2026-11-15) — end_crystal.rs

- 1 class, 0 named. `EndCrystalRenderer`: 26.2's `submitModel` call
  carries an extra trailing `null` argument (a 26.2-only parameter
  slot); 26.3's `shouldRender` additionally returns true when the
  crystal has a beam target (26.3-only — in 26.2 a culled crystal does
  not render, beam or not). The Rust has no beam-target render gate,
  matching 26.2; entity culling is the item-92 tick-position form.

## Item 317 no-change (2026-11-15) — client/gui/screens/worldselection

- 1 class, 0 named. `CreateWorldScreen`: `datapackWorldRegistries()`
  -> `datapackWorldgen()` rename only.

## Item 318 no-change (2026-11-15) — client/renderer/entity

- 1 class, 0 named. `DisplayRenderer`: 26.3's
  `getBoundingBoxForCulling(entity, partialTicks)` with
  `getInterpolatedBoundingBox` vs 26.2's `getBoundingBoxForCulling(entity)`
  with `getBoundingBoxForCulling()` — the item-92 tick-position culling
  form, already ported. `rotate` -> `mulPose` rename.

## Item 319 no-change (2026-11-15) — world/entity/projectile

- 2 classes, 0 named. `ProjectileDeflection`: 26.3's deflect lambdas
  take a `power` (Vec3) parameter; 26.2's take only (projectile,
  entity, random). REVERSE: 26.3 multiplies by `-power * 0.5` per
  component; 26.2 scales by -0.5. The Rust's `deflect_reverse` and
  `wind::reverse` both use the 26.2 -0.5 form, and 26.2's Breeze
  deflection is REVERSE (with the sound), not AIM_DEFLECT.
  `ShulkerBullet.onRedirectProjectile` (MOMENTUM_DEFLECT) is 26.3-only;
  26.2's shulker bullets have no such method, and the Rust correctly
  has none.

## Item 320 no-change (2026-11-15) — world/item/enchantment

- 3 classes, 0 named. `Enchantment`: `RegistryCodecs.holderSet` ->
  `homogeneousList` for `exclusive_set` (and `supported_items` /
  `primary_items` in the definition) and `RegistryFixedCodec.create`
  for `CODEC` — the same JSON shapes (a `#tag` string or a list), which
  the Rust's `parse_entry` already accepts in both forms.
  `Enchantment$Builder` drops the `Holder.direct` wrapping (26.3-only).

## Item 323 no-change (2026-11-15) — chunk/block_entity.rs

- 3 classes, 0 named. `TrialSpawnerConfig`: `RegistryCodecs.holder` ->
  `RegistryFileCodec.create` rename; the record field order is the same
  in both versions. `BlockEntity`: the two log lines swap their
  state/pos arguments — cosmetic only. `TrialSpawnerBlockEntity` diff
  is 10 lines of the same codec rename.

## Item 322 changed (2026-11-15) — level/portal.rs

- 2 classes, 0 named. `PortalShape.FRAME` is `state.is(#nether_portal_frame)`
  in 26.3 vs `state.is(Blocks.OBSIDIAN)` in 26.2. The tag file exists
  only in the 26.3 jar (46 bytes, contains only `minecraft:obsidian`);
  the real 26.2 `block-tags.json` (265 tags) has `minecraft:fire` but NOT
  `minecraft:nether_portal_frame`. The Rust's `portal_frame` (tag lookup
  via `b.tag(...)`, `None` -> false) was therefore always false under
  26.2 data — portal detection broken. Fixed in
  `crates/horizon-world/src/level/portal.rs` to
  `self.b.is_block(state, self.b.named.obsidian)` (named.obsidian exists
  at `level/data.rs:267`). `cargo check -p horizon-world` clean,
  rustfmt clean.
- Verification: the nether_portal harness had never run on 26.2 — it
  failed at case generation with `KeyError: ('minecraft:nether_portal',
  {'axis': 'z'})`. Root cause: the shared `States` helper in
  `tools/test_dispenser_parity.py` reads the state properties from
  `state[4]['properties']` (the 26.3 JSON form), but the 26.2 facts.json
  stores the NBT form (`Properties`). Fixed with a `state_properties`
  helper that accepts both forms (three call sites). Harness now runs
  and passes: **2626/2626 cases match (0 oracle errors)** — vanilla
  linked to an existing portal 725 times, built one 775 times, 1126 of
  them PortalShape frame searches.

## Batch of 12 rank-2 A items closed (2026-11-15) — 324, 325, 326, 327, 328, 329, 330, 331, 332, 334, 335, 336

All no-change. Rank 2 326/410 -> 338/410.

## Item 324 no-change (2026-11-15) — client-render/skins.rs

- 3 classes, 0 named. `PlayerSkinRenderCache$RenderInfo`: RenderPearl ->
  Blaze3D `GpuTextureView` import move only. `SkinManager` has no diff.
  The Rust's skins.rs is its own renderer.

## Item 325 no-change (2026-11-15) — client-render/vault.rs

- 3 classes, 0 named. `VaultBlock` carries a `MapCodec` in 26.2, removed
  in 26.3 (the item-309 pattern — the Rust does not use Mojang block
  codecs). `VaultRenderer`: `rotateDegrees` -> `mulPose` rename.
  `BuiltInLootTables`: four tables are 26.3-only (`till/rooted_dirt`,
  `barrels/abandoned_camp_barrel`, `chests/abandoned_camp_common_chest`,
  `chests/abandoned_camp_secret_chest`); no Rust references to them (the
  `abandoned_camp` hits in maps.rs are map decorations).

## Item 326 no-change (2026-11-15) — game/beacon.rs

- 4 classes, 0 named. `BeaconScreen` and its three button classes have
  no diff at all.

## Item 327 no-change (2026-11-15) — game/machines.rs

- 3 classes, 0 named. `DropperBlock` carries a `MapCodec` in 26.2,
  removed in 26.3 (item-309 pattern). `DispenserScreen` and
  `HopperScreen` have no diff.

## Item 328 no-change (2026-11-15) — game/ominous_item_spawner.rs

- 2 classes, 0 named. `OminousItemSpawner.getPistonPushReaction`:
  `IGNORE_ENTITY` (26.3) vs `IGNORE` (26.2) — the PushReaction rename
  already established (26.2 IGNORE = 26.3 IGNORE_ENTITY). The Rust's
  ominous spawner is a lightweight game struct, not an entity in the
  piston system, so the reaction never applies. Renderer:
  `rotateDegrees` -> `mulPose` rename.

## Item 329 no-change (2026-11-15) — shaders/pack.rs

- 1 class, 0 named. `Pack$ResourcesSupplier`: 26.3's
  `openMetadata`/`openResources` (Stream) vs 26.2's `openPrimary`/
  `openFull`. Mojang's pack-supplier interface; the Rust's pack loading
  is its own and does not implement it.

## Item 330 no-change (2026-11-15) — client/multiplayer

- 1 class, 0 named. `ClientSuggestionProvider`: 26.3 adds
  `getAvailablePostEffects` (post-effect suggestions, a 26.3-only
  shader feature) and `suggestRegistryElements` takes a `Predicate<E>`
  filter in 26.3 that 26.2's form lacks. The Rust has no such method.

## Item 331 no-change (2026-11-15) — tags

- 4 classes, 0 named. `ItemTags`: 26.3-only tags (WOOL_STAIRS/SLABS,
  POPLAR_LOGS, CONCRETE_STAIRS/SLABS via `BlockItemTags`;
  MUSHROOMS/CLONABLE_MAPS/EXTENDABLE_MAPS/CUSHIONS/DOUSES_CAMPFIRES/
  BREWING_POTION_INPUTS/FURNACE_FUEL_BOTTOM_TAKEABLE) and the 26.2-only
  `BREWING_FUEL` (bind; the Rust's brewing code already uses the
  `minecraft:brewing_fuel` tag). `FluidTags`: four 26.3-only tags
  (`axolotl_tries_to_find`, `dolphin_tries_to_find`,
  `frog_tries_to_find_land_near`, `entity_floatable`).
  `DamageTypeTags`/`EntityTypeTags`: 1 line each, tag-name renames.
  Tags are data-driven from the 26.2 tag files; the Rust already
  documents the 26.2 absences (`campfire_douse.rs`,
  `menu.rs` on `brewing_potion_inputs`).

## Item 332 no-change (2026-11-15) — world/item

- 3 classes, 0 named. `SwingAnimationType`: 26.3's `CODEC` is an
  `EnumCodec` with a `byName(name, default)` helper; 26.2's is a plain
  `Codec`. The Rust references the type by ordinal only (STAB = 2) —
  no wire effect. `ItemStackTemplate.fromNonEmptyStack` is 26.3-only;
  the Rust's `Template::from_non_empty_stack` is an internal helper
  with the same semantics, used only by its own fixtures. `JukeboxSong`:
  `RegistryCodecs.holder` -> `RegistryFixedCodec.create` rename.

## Item 334 no-change (2026-11-15) — client-render/widgets.rs

- 2 classes, 0 named. `RecipeButton`: `button() == 1 || == 3` (26.3
  SDL) vs `== 0 || == 1` (26.2 GLFW) — the same left/right clicks
  under the item-310 numbering split. `CursorTypes`: SDL standard
  cursor ids (0, 1, 3, 11, 8, 7, 9, 10) vs GLFW ids (221185+). The
  Rust's cursor is symbolic (`Cursor::Arrow/PointingHand/NotAllowed`),
  with no numeric ids.

## Item 335 no-change (2026-11-15) — game/sheep_dye.rs

- 1 class, 0 named. `DyeItem.tryApplyToSign`: 26.3 takes a
  `SignTextSlot`; 26.2 takes a boolean `isFrontText`. The Rust's
  `use_sign_applicator` already uses the 26.2 form (`facing_sign_front`
  -> `sign.front` / `sign.back`).

## Item 336 no-change (2026-11-15) — game/trial_spawner.rs

- 1 class, 0 named. `TrialSpawner`: 26.3 sets the OMINOUS flag with
  `level.setBlockAndUpdate`; 26.2 uses `level.setBlock(pos, state, 3)`
  (the 26.2 form already documented in `chiseled_bookshelf.rs`).
  `overrideEntityToSpawn(TypedEntityData, level)` is 26.3-only. The
  Rust's trial spawner reads the ominous flag from the block state and
  has no `setBlock` call for it; its config override is its own.

## CI run 43646940294 (dispatched 2026-11-15, tree d2aa9a03, 338/410)

Dispatched on the ~50-item cadence (previous run 43609469257 was the
285/410 tree). Profile pending — the run takes ~2h; porting continues
while it runs. Artifacts to `~/workspace/mcir-ci/26.2-338/` when done.

## Batch of 12 rank-2 A items closed (2026-11-15) — 337, 338, 339, 340, 341, 343, 344, 345, 346, 347, 348, 349

All no-change. Rank 2 338/410 -> 350/410. CI run 43646940294 is in
flight on the 338/410 tree.

## Item 337 no-change (2026-11-15) — game/wind.rs

- 1 class, 0 named. `FenceGateBlock` carries a `MapCodec` (with a
  `wood_type` field) in 26.2, removed in 26.3 (the item-309 pattern —
  the Rust does not use Mojang block codecs).

## Item 338 no-change (2026-11-15) — lan/hub/chat.rs

- 1 class, 0 named. `SignedMessageBody$Packed`: 26.3's `STREAM_CODEC`
  (ByteBuf) vs 26.2's FriendlyByteBuf constructor/`write` — the same
  wire format (content utf8 256, instant, long salt, lastSeen Packed),
  the item-301 pattern.

## Item 339 no-change (2026-11-15) — climbing_fixture.rs

- 1 class, 0 named. `TrapDoorBlock` carries a `MapCodec` (with a
  `block_set_type` field) in 26.2, removed in 26.3 (item-309 pattern).

## Item 340 no-change (2026-11-15) — level/mod.rs

- 2 classes, 0 named. `ComparatorBlock` carries a `MapCodec` in 26.2,
  removed in 26.3; `BaseEntityBlock` declares the abstract `codec()`
  in 26.2 (item-309 pattern).

## Item 341 no-change (2026-11-15) — client/gui/screens/inventory

- 2 classes, 0 named. `AnvilScreen` and `CraftingScreen` have no diff
  at all.

## Item 343 no-change (2026-11-15) — client/resources

- 1 class, 0 named. `ClientPackSource`: 26.3's
  `createVanillaPack(ResourcesSupplier)` vs 26.2's
  `createVanillaPack(PackResources)` + `fixedResources` — the item-329
  pack-supplier rework. The Rust's pack loading is its own.

## Item 344 no-change (2026-11-15) — blaze3d/platform/cursor

- 1 class, 0 named. `CursorType.select`: SDL (`SDLMouse`) in 26.3 vs
  GLFW (`glfwSetCursor`/`glfwCreateStandardCursor`) in 26.2 — the
  item-310/334 platform split. The Rust uses winit (GLFW-style).

## Item 345 no-change (2026-11-15) — world/entity/vehicle/minecart

- 1 class, 0 named. `AbstractMinecart`: 26.3's
  `createInterpolationHandler()` (LinearInterpolationHandler or NO_OP
  for experimental movement) vs 26.2's `getInterpolation()` from the
  behavior (the 3-step handler). The Rust's minecart movement is
  ported from `NewMinecartBehavior` (per-step rotation and
  interpolation weight) and renders at the tick position (the item-92
  form); it does not implement 26.3's LinearInterpolationHandler.

## Item 346 no-change (2026-11-15) — world/level/block/piston

- 2 classes, 0 named. `MovingPistonBlock` carries a `MapCodec` in
  26.2, removed in 26.3 (item-309 pattern). `PistonBaseBlock$1` is the
  compiler switch map over `PushReaction` — IMMOVEABLE/POPPED/PUSH
  (26.3) vs BLOCK/DESTROY/PUSH_ONLY (26.2), the established enum
  rename; the Rust's piston logic is ported against the 26.2 behaviour.

## Item 347 no-change (2026-11-15) — world/level/levelgen/structure/pools

- 2 classes, 0 named. `JigsawPlacement$Placer`: 26.3 reads the
  `JigsawBlockInfo` record's `.pos()`/`.state()` directly; 26.2 reads
  them through `StructureBlockInfo` (`.info()`) — the item-214 rename
  story. The Rust's jigsaw code uses its own `JigsawInfo` struct with
  `pos`/`state` fields. `StructureTemplatePool`: `RegistryCodecs.holder`
  -> `RegistryFileCodec.create` rename.

## Item 348 no-change (2026-11-15) — client-render/breaking.rs

- 1 class, 0 named. `SheetedDecalTextureGenerator`: 26.3 adds `setUv3`
  and the static `setSheetedDecalUv` (the uv3/glint path, used by
  `putBakedQuadWithGlint`). 26.2's `setNormal` computes the decal UV
  inline and calls `delegate.setUv` (uv0) with the identical transform
  (rotateY(PI), rotateX(-PI/2), direction rotation, negate) — the
  Rust's breaking.rs implements exactly that transform. The 26.3-only
  part is the uv3 slot, which the Rust does not use.

## Item 349 no-change (2026-11-15) — gfx/lightning/tests

- 1 class, 0 named. `VertexConsumer`: 26.3 adds `setUv3` and the
  default `putBakedQuadWithGlint` (the uv3/glint path from item 348).
  Mojang's vertex-consumer interface; the Rust's is its own wgpu code.

## Batch of 12 rank-2 A items closed (2026-11-15) — 350, 351, 352, 353, 354, 358, 359, 360, 363, 364, 365, 366

All no-change. Rank 2 350/410 -> 362/410. CI run 43646940294 still in
flight on the 338/410 tree.

## Item 350 no-change (2026-11-15) — game/paintings.rs

- 2 classes, 0 named. `PaintingRenderer`: `rotateDegrees` -> `mulPose`
  rename. `PaintingVariant`: `RegistryCodecs.holder` ->
  `RegistryFixedCodec.create` rename.

## Item 351 no-change (2026-11-15) — gossip.rs

- 1 class, 0 named. `GossipContainer.transferFrom` returns the picked
  count (`int`) in 26.3 and `void` in 26.2. The Rust's `transfer_from`
  returns the count, but its only non-test caller (villager.rs:675)
  discards it — the 26.2 void form is honoured in effect; the return
  is a test-only convenience. The transfer itself (selection, decay
  per transfer, discard threshold) is identical.

## Item 352 no-change (2026-11-15) — mobs/breeze.rs

- 1 class, 0 named. `Breeze.PROJECTILE_DEFLECTION`: 26.3's 4-arg
  lambda (with `power`) vs 26.2's 3-arg form — the item-319 story.
  The Rust's `deflect(REVERSE)` already uses the 26.2 -0.5-per-axis
  form (arrows.rs).

## Item 353 no-change (2026-11-15) — mobs/landing.rs

- 1 class, 0 named. `HayBlock` carries a `MapCodec` in 26.2, removed
  in 26.3 (item-309 pattern).

## Item 354 no-change (2026-11-15) — mobs/llama.rs

- 1 class, 0 named. `Llama`: 26.3's `rangedAttackUncertainty(level)`
  returns 10.0F; 26.2 inlines the literal 10.0F in the spit call. The
  Rust's spit passes the literal `10.0` — the same value.

## Item 358 no-change (2026-11-15) — horizon-core/paletted.rs

- 2 classes, 0 named. `BitStorage.fill(int)` is 26.3-only (interface
  method + `SimpleBitStorage` implementation, the bit-replication
  fill). The Rust's paletted storage has no `fill` method.

## Item 359 no-change (2026-11-15) — world/dimension_type.rs

- 1 class, 0 named. `DimensionType`: `RegistryCodecs.holder` ->
  `RegistryFileCodec.create` and `holderSet` -> `homogeneousList` for
  `infiniburn`/`timelines` — the same JSON shapes. The Rust's
  `DimensionFacts.infiniburn` reads the tag string
  (`#minecraft:infiniburn_overworld`/`_nether` defaults).

## Item 360 no-change (2026-11-15) — level/data.rs

- 1 class, 0 named. `WallBlock` carries a `MapCodec` in 26.2, removed
  in 26.3 (item-309 pattern).

## Item 363 no-change (2026-11-15) — advancements

- 1 class, 0 named. `AdvancementHolder`: 26.3's
  `register(BootstrapContext)` (server-side bootstrap) vs 26.2's
  `LIST_STREAM_CODEC` (the packet list form). The Rust decodes
  `ClientboundUpdateAdvancementsPacket` as a list — the 26.2 form.

## Item 364 no-change (2026-11-15) — client/gui/screens

- 2 classes, 0 named. `DeathScreen` has no diff.
  `LoadingOverlay$LogoTexture`: `asResourceManager()` (26.3) vs
  `asProvider()` (26.2) — rename only.

## Item 365 no-change (2026-11-15) — client/renderer/state

- 1 class, 0 named. `GameRenderState`: 26.3 adds the
  `requestedPostEffects` list and the `shouldRenderLevel` /
  `readyForLevelRendering` flags; 26.2 has the `useShaderTransparency()`
  method instead. The post-effect/transparency rework; the Rust's
  render state is its own wgpu code.

## Item 366 no-change (2026-11-15) — network/syncher

- 2 classes, 0 named. `EntityDataSerializers`: 26.3 registers a
  `DYE_COLOR` serializer at id 43; 26.2's table ends at `HUMANOID_ARM`
  (id 42). The Rust's table has the same 0-42 order and the extra
  DYE_COLOR arm at 43 — unreachable from a 26.2 server, which never
  sends id 43, so no 26.2 wire difference.
  `SynchedEntityData$Builder`: `id >= length` (26.3) vs `id > length`
  (26.2) in the bounds check — server-side vanilla behaviour; the
  Rust client never builds it (it decodes `ClientboundSetEntityDataPacket`
  entries by index).

## Batch of 12 rank-2 A items closed (2026-11-15) — 367, 368, 369, 370, 371, 372, 373, 375, 376, 377, 378, 379

Eleven no-change, one changed (371). Rank 2 362/410 -> 374/410. CI run
43646940294 still in flight on the 338/410 tree.

## Item 367 no-change (2026-11-15) — sounds

- 2 classes, 0 named. `SoundEvent`: `RegistryCodecs.holder` ->
  `RegistryFileCodec.create` rename. `SoundEvents`: 26.3 registers some
  sounds with `registerForHolder` (returning `Reference<SoundEvent>`)
  where 26.2 uses plain `register` — a holder-vs-direct distinction
  with no wire effect. The 26.3-only sounds (CUSHION_*, SHELF_MUSHROOM_*,
  POPLAR_LEAVES_*, STRAW_BED_*, RED_SHRUB_*) have no Rust references;
  the Rust's sound table is data-driven from the 26.2 registry.

## Item 368 no-change (2026-11-15) — world/attribute/modifier

- 1 class, 0 named. `AttributeModifier`: 26.3's ColorModifier libraries
  are generic over `Vector3fc`/`Vector4fc` (RGB/ARGB variants); 26.2's
  are `Integer`-based with the plain ADD/SUBTRACT/BLEND_TO_GRAY/ALPHA_BLEND
  names. 26.3 adds `MOB_SPAWN_SETTINGS_LIBRARY` and `listLibrary`
  (ListModifier) — both 26.3-only. The Rust does not port the
  color-modifier libraries.

## Item 369 no-change (2026-11-15) — world/entity/monster/cubemob

- 1 class, 0 named. `MagmaCube`: `canDealDamage` (26.3) vs
  `isDealsDamage` (26.2) — a rename. 26.2's returns `isEffectiveAi()`;
  the Rust's `can_deal_damage` (cube.rs) returns true for magma cubes
  and the call sites check `m.no_ai` separately, reproducing the same
  effective behaviour.

## Item 370 no-change (2026-11-15) — world/entity/monster/hoglin

- 1 class, 0 named. `Hoglin.blockedByItem` takes a `fullyBlocked`
  parameter in 26.3 (4-arg) that 26.2's 3-arg form lacks, and
  `handleEntityEvent` carries a `@Value` annotation in 26.3 only. The
  Rust has no `blocked_by_item` implementation for hoglins.

## Item 371 changed (2026-11-15) — world/inventory

- 2 classes, 0 named. `MerchantResultSlot.mayPickup` is 26.3-only: it
  blocks taking the result unless the active offer is satisfied by the
  buy slots. 26.2's slot has no such override, so `Slot.mayPickup`
  (true) applies — the take is allowed even when the offer is not
  satisfied, and `onTake` then runs `offer.take` (which fails without
  consuming) plus `overrideXp` — which is an empty method on
  `AbstractVillager`, so no XP either. The Rust's `may_pickup` had the
  26.3 gate (`Role::MerchantResult` arm requiring `satisfied_by`).
  Fixed in `crates/horizon-client/src/menu.rs`: the arm now returns
  true, with the 26.2 form documented; `merchant_take` already matches
  26.2's `onTake` (nothing happens when `take` fails). `StonecutterMenu`:
  `player.drop(stack, false, Prediction.PREDICTED)` in 26.3 vs
  `player.drop(stack, false)` in 26.2 — `Prediction` is 26.3-only; the
  Rust's stonecutter uses its own `drop_stack` with no prediction
  parameter.

## Item 372 no-change (2026-11-15) — beehive.rs

- 1 class, 0 named. `HorizontalDirectionalBlock` declares the abstract
  `codec()` in 26.2 (item-309 pattern).

## Item 373 no-change (2026-11-15) — container_screen.rs

- 1 class, 0 named. `AbstractRecipeBookScreen.isInputCaptured` is
  26.3-only (`Screen.isInputCaptured` does not exist in 26.2).

## Item 375 no-change (2026-11-15) — dialog.rs

- 1 class, 0 named. `Dialog`: `RegistryCodecs.holder` ->
  `RegistryFileCodec.create` and `holderSet` -> `homogeneousList` for
  `LIST_CODEC` — the same JSON shapes.

## Item 376 no-change (2026-11-15) — game/campfire.rs

- 1 class, 0 named. `RecipeType.BREWING` is 26.3-only. The Rust has no
  BREWING recipe type (26.2 brewing is via the brewing stand, not a
  recipe type).

## Item 377 no-change (2026-11-15) — game/entity.rs

- 1 class, 0 named. `Entity$1` is the compiler switch map over
  `MoveSimulationType` (SERVER_AND_CLIENT / AUTHORITATIVE_SIDE /
  AUTHORITATIVE_SIDE_AND_SERVER) — the 26.3 movement-simulation enum.
  The Rust's server-side player travel (`server_travel`, fall-distance
  reset on climbables) matches 26.2's actual behaviour; the two
  comments citing the 26.3 enum name are citations only.

## Item 378 no-change (2026-11-15) — game/game_mode.rs

- 1 class, 0 named. `InventoryScreen` has no diff at all.

## Item 379 no-change (2026-11-15) — game/lan_mobs.rs

- 1 class, 0 named. `OwnerHurtByTargetGoal`: 26.3 gates the retaliation
  on `!lastDamageSource.is(DamageTypeTags.NO_WOLF_RETALIATION)`; 26.2
  has no such check. The Rust's `OwnerHurtBy.can_use` (tame.rs) checks
  only the owner's `hurt_by` and `wants_to_attack` — the 26.2 form.
  No `#no_wolf_retaliation` reference exists in the Rust.

## Batch of 12 rank-2 A items closed (2026-11-15) — 380, 381, 382, 383, 384, 385, 386, 387, 388, 389, 390, 391

Eleven no-change, one changed (387). Rank 2 374/410 -> 386/410. CI run
43646940294 still in flight on the 338/410 tree.

## Item 380 no-change (2026-11-15) — game/mob_gear.rs

- 1 class, 0 named. `EnchantmentsByCostWithDifficulty`: `holderSet` ->
  `homogeneousList` for the `enchantments` field — the same JSON shape.

## Item 381 no-change (2026-11-15) — game/native_panel_recipes.rs

- 1 class, 0 named. `SlotDisplay$AnyFuel`: 26.3 reads fuel items from
  `DataComponents.COOKING_FUEL` via the item registry; 26.2 reads them
  from `FuelValues` (`SlotDisplayContext.FUEL_VALUES`). The Rust's
  AnyFuel is every item with a burn time from the 26.2 fuel data
  (`burn_time`), grouped by burn time — the 26.2 source.

## Item 382 no-change (2026-11-15) — game/remote_mob_data.rs

- 1 class, 0 named. `RopesLayer`: 26.2's `submitModel` call carries the
  extra trailing `null` argument (the item-315 pattern).

## Item 383 no-change (2026-11-15) — game/storage.rs

- 1 class, 0 named. `ShulkerBoxScreen` has no diff at all.

## Item 384 no-change (2026-11-15) — mobs/camel.rs

- 1 class, 0 named. `Camel.mobInteract`: 26.3's else-chain vs 26.2's
  flattened fall-through — the same outcomes (non-baby with <2
  passengers rides and returns CONSUME in both; a baby holding a golden
  dandelion goes to `super.mobInteract`, otherwise FAIL). The Rust's
  camel interact goes through the shared mount path with the same
  outcomes.

## Item 385 no-change (2026-11-15) — mobs/finalize.rs

- 1 class, 0 named. `VillagerType`: `RegistryCodecs.holder` ->
  `RegistryFixedCodec.create` rename.

## Item 386 no-change (2026-11-15) — mobs/nav.rs

- 1 class, 0 named. `GroundPathNavigation`: 26.3 checks
  `isInFloatableFluid()` and `state.getFluidState().is(#entity_floatable)`;
  26.2 checks `isInWater()` and `state.is(Blocks.WATER)`. The Rust's
  `surface_y` uses `fluid == Fluid::Water` and `navigation_in_water()`
  — the 26.2 forms. `#entity_floatable` is 26.3-only (item 331).

## Item 387 changed (2026-11-15) — mobs/zombie_villager.rs

- 1 class, 0 named. `ZombieVillager` curing: 26.3 checks
  `state.is(#speeds_up_zombie_villager_curing)`; 26.2 checks
  `state.is(Blocks.IRON_BARS) || state.getBlock() instanceof BedBlock`.
  The tag file exists only in the 26.3 jar (iron_bars + #beds); the
  26.2 `block-tags.json` (265 tags) does not have it — so the Rust's
  `has_tag` lookup was always false and the curing speedup never fired.
  Fixed in `crates/horizon-client/src/mobs/zombie_villager.rs`: the
  check is now iron bars or any bed (via the `#minecraft:beds` tag,
  which exists in 26.2 with all 16 beds), extracted as
  `speeds_up_curing`. Regression test
  `iron_bars_and_beds_speed_a_cure` passes with the fix and fails with
  the old tag lookup.

## Item 388 no-change (2026-11-15) — mod_content_biomes.rs

- 1 class, 0 named. `FeatureSorter`: `Reference2IntMap` (26.3) vs
  `Object2IntMap` (26.2) for the feature index — identity vs equals
  map. The Rust has no FeatureSorter equivalent (placed features are
  looked up by name).

## Item 389 no-change (2026-11-15) — horizon-items/enchant.rs

- 1 class, 0 named. `Enchantments` bootstrap: the same conditions and
  values built through different Java APIs (`TagPredicate.is(damageTypes,
  tag)` vs `TagPredicate.is(tag)`; `ContextFloatProviders` vs
  `EnchantmentLevelProvider`; `matchesTag(Direction.UP, ...)` vs
  `matchesTag(Vec3i(0,1,0), ...)`; `BlockStateProvider.holderOf` vs
  `simple`). The Rust reads enchantments from the registry data, not
  from the bootstrap Java — the data is the same.

## Item 390 no-change (2026-11-15) — damage_catalog_probe.rs

- 1 class, 0 named. `DamageType`: `RegistryCodecs.holder` ->
  `RegistryFixedCodec.create` rename.

## Item 391 no-change (2026-11-15) — attributes/entity.rs

- 1 class, 0 named. `DefaultAttributes`: `EnderMan` (26.2) vs
  `Enderman` (26.3) — the class rename.

## Batch of 23 rank-2 A items closed (2026-11-15) — 392, 394, 395, 396, 398, 399, 400, 401, 402, 403, 404, 405, 406, 408, 409, 410, 411, 412, 413, 414, 415, 416, 417

Twenty-two no-change, one changed (402). Rank 2 386/410 -> 409/410.
Bucket A is now complete; only item 127 (parked, D bucket) remains open
in rank 2. CI run 43646940294 still in flight on the 338/410 tree.

## Item 392 no-change (2026-11-15) — world/chunk.rs

- 1 class, 0 named. `ZeroBitStorage.fill(int)` is 26.3-only (the item-358
  pattern). The Rust's paletted storage has no `fill` method.

## Item 394 no-change (2026-11-15) — worldgen/structure/processor.rs

- 1 class, 0 named. `IntProviders`: 26.3 registers
  `very_biased_to_bottom`. The Rust's height provider handles that type,
  but it is only invoked when the data names it — and 26.2's data never
  does, so the extra handler is dead code for 26.2 (the DYE_COLOR
  pattern from item 366).

## Item 395 no-change (2026-11-15) — client/gui/components/events

- 1 class, 0 named. `GuiEventListener.capturesInput()` is 26.3-only
  (the item-373 pattern).

## Item 396 no-change (2026-11-15) — client/gui

- 1 class, 0 named. `Gui$1` (synthetic) has no diff at all.

## Item 398 no-change (2026-11-15) — client/renderer/block/dispatch

- 1 class, 0 named. `BlockStateModelDispatcher$SimpleModelSelectors`:
  the model-selection loop is identical; only the log line differs
  (26.3 includes the exception message). The Rust's model dispatcher is
  its own.

## Item 399 no-change (2026-11-15) — realmsclient/client/worldupload

- 1 class, 0 named. `RealmsCreateWorldFlow`: a log-message format
  difference only. The Rust does not run Realms.

## Item 400 no-change (2026-11-15) — core/particles

- 1 class, 0 named. `ParticleTypes`: the `red/orange/yellow_poplar_leaves`
  particles are 26.3-only, and the particle-options codec dispatch gains
  the `"type"` key in 26.2. The Rust's `ParticleType` enum carries the
  three poplar variants, but particle ids resolve by NAME through the
  adapter's remap tables (data-driven from the 26.2 registry), and no
  26.2 data ever names them — unreachable on 26.2 (the DYE_COLOR pattern).

## Item 401 no-change (2026-11-15) — core/registries

- 1 class, 0 named. `Registries`: the 26.3 type-registry rework
  (`MapCodec<? extends X>` type registries vs 26.2's instance
  registries) plus 26.3-only keys (`block_transformer`,
  `context_key_set`, `slot_source`, `context_float/int_provider`,
  `worldgen/carver_type`, `worldgen/feature_type`,
  `worldgen/material_condition/rule_type`, `worldgen/noise` as
  NormalNoise). The Rust loads registries by data path from the 26.2
  files; the 26.3-only datapack registries (`context_int_provider`,
  `context_float_provider`, `slot_source`) do not exist in 26.2's data
  at all (0 entries in the 26.2 jar vs 37 in 26.3's), and the Rust's
  loaders no-op on their absence (`load_dir` returns on a missing
  directory). No 26.2 wire or data difference.

## Item 402 changed (2026-11-15) — server/dedicated

- 1 class, 0 named. `DedicatedServerProperties`: `white-list` defaults
  to `true` in 26.3 and `false` in 26.2. The Rust's
  `horizon-server-files/src/properties.rs` used the 26.3 default
  (`r.boolean("white-list", true)`), which flows into
  `lists.use_whitelist` — a dedicated server started without
  `white-list` in server.properties would enforce the whitelist in the
  Rust but not in vanilla 26.2. Fixed to `false`, with the 26.2 default
  documented. The existing `defaults_match_vanilla_init_settings` test
  pinned the 26.3 value in its stored-output suffix; updated to
  `white-list=false` with an explicit `assert!(!p.white_list)`.

## Item 403 no-change (2026-11-15) — server/network

- 1 class, 0 named. `ServerGamePacketListenerImpl$1` is the compiler
  switch map over `ServerboundPlayerActionPacket.Action` — the ordinals
  differ by one for CHANGE_DESTROY_DIRECTION and friends, the item-12
  PlayerAction ordinal shift, already ported (client sends + host
  dispatch to 26.2 ordinals).

## Item 404 no-change (2026-11-15) — stats

- 1 class, 0 named. `Stats`: `SLEEP_IN_STRAW_BED` is 26.3-only. No Rust
  reference.

## Item 405 no-change (2026-11-15) — util/random

- 1 class, 0 named. `WeightedList$Builder.addAll` is 26.3-only. The
  Rust's weighted list has no `add_all`.

## Item 406 no-change (2026-11-15) — world/attribute

- 1 class, 0 named. `EnvironmentAttributeReader`: 26.3 reads the ORIGIN
  with `getOptional` and falls back to the attribute default when it is
  absent; 26.2 reads it with `getParameter` (throws when absent). The
  Rust's environment system takes a position directly for positional
  attributes (no loot-context path), so the optional-vs-required
  distinction has no Rust surface.

## Item 408 no-change (2026-11-15) — world/entity/ai/behavior

- 1 class, 0 named. `MeleeAttack`: `swingForAttack` (26.3) vs `swing`
  (26.2). The Rust's melee attack uses `swing` (ai.rs).

## Item 409 no-change (2026-11-15) — world/entity/animal/goat

- 1 class, 0 named. `GoatAi`: only the IDLE activity's memory
  requirements changed — 26.3 wants `RAM_COOLDOWN_TICKS` +
  `LONG_JUMP_COOLDOWN_TICKS` VALUE_PRESENT; 26.2 wants `RAM_TARGET` +
  `LONG_JUMP_MID_JUMP` VALUE_ABSENT. The RAM and LONG_JUMP activities
  are identical in both. The Rust has no brain activities; its goat
  gating faithfully implements 26.2's order — RAM first (its cooldown
  absent), LONG_JUMP when RAM is on cooldown (`ram_cooldown > 0`) with
  TEMPTING_PLAYER/BREED_TARGET/WALK_TARGET absent and its own cooldown
  absent (`jump_cooldown(m).is_none()` in frog.rs).

## Item 410 no-change (2026-11-15) — world/entity/animal/nautilus

- 1 class, 0 named. `AbstractNautilus`: `calculateViewVector` is a
  static call in 26.3 and an instance call (`this.calculateViewVector`)
  in 26.2 — the same math.

## Item 411 no-change (2026-11-15) — world/entity/decoration

- 1 class, 0 named. `Mannequin`: 26.2's `aiStep` calls
  `updateSwingTime()`; 26.3's does not. The Rust's mob tick calls
  `tick_swing(m)` for every mob including mannequins (ai.rs:793) — the
  26.2 form.

## Item 412 no-change (2026-11-15) — world/entity/item

- 1 class, 0 named. `ItemEntity.getMoveSimulationType` returning
  SERVER_AND_CLIENT is 26.3-only (the item-377 pattern).

## Item 413 no-change (2026-11-15) — world/item/consume_effects

- 1 class, 0 named. `RemoveStatusEffectsConsumeEffect`: `holderSet` ->
  `homogeneousList` for the `effects` field — the same JSON shape.

## Item 414 no-change (2026-11-15) — world/level/block/entity/vault

- 1 class, 0 named. `VaultBlockEntity$Server`: 26.3 calls
  `setBlockAndUpdate`; 26.2 calls `setBlock(pos, state, 3)` — and
  `Level.setBlockAndUpdate` IS `setBlock(pos, state, 3)` in both
  versions, so the two forms are identical. The Rust's `set_vault_state`
  goes through the general `level_set` path.

## Item 415 no-change (2026-11-15) — world/level/gameevent

- 1 class, 0 named. `GameEvent`: `RegistryCodecs.holder` ->
  `RegistryFixedCodec.create` rename.

## Item 416 no-change (2026-11-15) — world/level/levelgen/structure/pieces

- 1 class, 0 named. `StructurePieceSerializationContext`:
  `getStructureTemplateManager` (26.3) vs `getStructureManager` (26.2)
  — rename only.

## Item 417 no-change (2026-11-15) — world/level/storage/loot/parameters

- 1 class, 0 named. `LootContextParams`: the `CONTAINER` ContextKey
  (SlotProvider) is 26.3-only. The Rust's loot context has no CONTAINER
  key (it uses ORIGIN and BLOCK_ENTITY).

## Item 127 — the one remaining rank-2 item (2026-11-15)

Rank 2 stands at 409/410 with bucket A complete. Item 127
(`world/level/levelgen/synth`, 12 classes, 0 named) is the only open
rank-2 item, and it is NOT closable as no-change: the Rust's noise
engine mirrors 26.3's noise model, and 26.2's is a different one.

The 26.2 synth package has 9 classes: BlendedNoise, ImprovedNoise,
NoiseUtils, NormalNoise (+NoiseParameters), PerlinNoise,
PerlinSimplexNoise, SimplexNoise. The 26.3 package has 19: it adds
BlendedNoise$FbmSet, GradientNoise (+Gradient), LegacyFbmInitializer,
Noise, NoiseStack (+Builder/Layer/Perlin/SmearedPerlin),
NormalNoise$Builder/Normalization/OctaveInfo/Parameters and
SmearedPerlinNoise, and removes ImprovedNoise and PerlinSimplexNoise.
26.2's NormalNoise is built from NoiseParameters ({firstOctave,
amplitudes} only); 26.3's is built from a builder around a NoiseStack.

The Rust's noise engine (`crates/horizon-core/src/noise/`, 1,776 lines:
mod.rs, normal.rs, perlin.rs, simplex.rs, stack.rs, volume.rs) is the
26.3 model (NoiseStack, DensityVolume). The 26.3 oracle
(`VanillaNoiseOracle.java`, 149 lines) is built entirely on 26.3-only
classes — GradientNoise, Noise, NoiseStack, NormalNoise (builder form),
PerlinNoise/SimplexNoise/SmearedPerlinNoise, plus DensityBuffer and
DensityVolume from the `densityfunction` package, which does not exist
in 26.2 — so it does not compile against the 26.2 jar at all.

Closing 127 needs, in order: (1) port the noise engine to the 26.2
model (ImprovedNoise, PerlinSimplexNoise, the old NormalNoise and
BlendedNoise); (2) rewrite VanillaNoiseOracle.java against 26.2's
synth API; (3) rewrite the driver (`tools/test_noise_parity.py`) and
the Rust probe to the 26.2 cases; (4) re-run and clear the 792-fixture
noise_based_count placement divergence this item owns (vanilla 26.2's
`Biome.BIOME_INFO_NOISE`, old Perlin-based NormalNoise, vs the Rust
engine mirroring 26.3's noise system). That is a large, high-risk job
— a wrong noise port regresses chunk_generation, the biome harnesses
and placement — and it needs its own session with careful verification,
not a rushed close. It is recorded here as the precise blocker for
410/410, not hidden.

## CI run 43653524277 dispatched on the 409/410 tree (2026-11-15)

Continuation agent's first act: `gh workflow run port-verify.yml -f
version=26.2 --ref behaviour/26.2` on dbeb88e3 (rank 2 409/410), run
43653524277, in progress. Run 43646940294 is still in flight on the
older 338/410 tree (d2aa9a03); this run measures the 409/410 state.
Porting continues while CI runs.

## Item 127 — noise-model port, in progress (2026-11-15)

The 26.2 synth package (9 classes) is the classic double-precision model:
ImprovedNoise, PerlinNoise (octave FBM, new + legacy init), SimplexNoise
(no offsets in `getValue`), PerlinSimplexNoise, NormalNoise over
NoiseParameters `{firstOctave, amplitudes}`, BlendedNoise, NoiseUtils.
Verified against the 26.2 jars: `Biome.BIOME_INFO_NOISE` is a
PerlinSimplexNoise (seed 2345, octaves [0]) — the earlier placement note
calling it an "old Perlin-based NormalNoise" was wrong; TEMPERATURE_NOISE
(seed 1234, [0]) and FROZEN_TEMPERATURE_NOISE (seed 3456, [-2,-1,0]) are
PerlinSimplexNoise too. `WorldgenRandom` delegates `next(bits)` straight
through to the wrapped legacy source, so the Biome noises run on plain
LegacyRandom streams.

Design: the float sampler engine (`noise::{PerlinNoise, SimplexNoise,
NoiseStack, NormalNoise}`) stays — it is Horizon's own scaffolding, its
octave structure and seed consumption already match 26.2, and the density
graph built on it is f32 end to end (converting that engine is the
programme-level double rework recorded under item 115, not this item).
What this item adds is the faithful f64 port alongside it, used where
vanilla 26.2 computes in double and the value feeds a discrete decision.

(a) `crates/horizon-core/src/noise/synth.rs` (new, ~1,300 lines):
ImprovedNoise, PerlinNoise, SimplexNoise, PerlinSimplexNoise,
NormalNoise + NoiseParameters, BlendedNoise, NoiseUtils — all f64, with
Java's wrapping int arithmetic, `Mth.lfloor` (long floor) in `wrap`,
the `float` literal `1.0E-7F` in the smear, and the `float` literal
`9.223372E18F` in the positive-octave seed. Six unit tests pin the
shared seeding with the float engine, wrap edge cases (including the
i64-saturation divergence from the float wrap at huge magnitudes), the
single-octave PerlinSimplexNoise identity, the old value-factor formula,
BlendedNoise construction, and legacy-init throw order. `cargo test -p
horizon-core --lib noise::` 12/12; fmt + clippy clean for the new file.

(b)+(c) Noise oracle, probe and driver rewritten for 26.2 (2026-11-15):
`VanillaNoiseOracle.java` now drives the 26.2 synth API (ImprovedNoise,
PerlinNoise, SimplexNoise, PerlinSimplexNoise, NormalNoise, BlendedNoise,
NoiseUtils) with `SharedConstants.tryDetectVersion()` + `Bootstrap
.bootStrap()` at startup — touching `DensityFunction.FunctionContext`
for `BlendedNoise.compute` initializes `DensityFunctions`, which needs
the registries. `crates/horizon-core/src/bin/noise_parity.rs` drives
the synth module over the same protocol; `tools/test_noise_parity.py`
has a 26.2 corpus (wrap incl. long-floor saturation, bias, improved
+ local queries, perlin new/legacy direct + octave-set forms with
error draw-state cases, simplex, perlin-simplex incl. positive-octave
seeding, normal incl. legacy Nether biome, blended seeded + unseeded).
`python3 tools/test_noise_parity.py` PASSES: 6,032 output lines match
Rust bit-exactly on BOTH the client and server jars. Two driver bugs
found on the way: the legacy PerlinNoise init fails (count check) when
a nonzero amplitude sits above the zero octave — e.g. firstOctave 0
with [1.0, 1.0] — so the driver now mirrors the throw order instead
of assuming only firstOctave-1 cases fail.

(d) Biome noises to the f64 synth port (2026-11-15): `TemperatureNoises`
is now three `synth::PerlinSimplexNoise` (1234/[0], 3456/[-2,-1,0],
2345/[0]) instead of the float simplex/stack; `biome_info_noise`
returns f64; the frozen modifier and height-adjusted temperature compute
in double with vanilla's float casts at the exact points
(`(float)(noise * 8.0)` in the height adjustment); placement's
NoiseBasedCount/NoiseThresholdCount use the double directly. VERIFIED
2026-11-15: placement harness re-run, fixture-by-fixture comparison of
the 18,780 `tail`-delimited blocks — mismatches 882 → 90, and all 792
noise_based_count fixtures now match; the only remaining 90 are
count_on_every_layer (the separate pull-order issue, not this item's
scope). Item 127 CLOSED with this step: rank 2 is 410/410.

## CI run 43646940294 (2026-11-15) — 338/410 tree d2aa9a03

**5 pass / 0 oracle / 12 fail** — first_person, food, lan, light,
resource_pack (same five as 43609469257). D1 pass; D2 FAIL on a real
compile error in a test target: `rule_test_bounds.rs:124` still called
the 3-arg `RuleTest::test(state, [0; 3], random)` after item 214 changed
the signature to the 26.2 2-arg form — E0061, `horizon-world` test
`rule_test_bounds` would not compile. Fixed on behaviour/26.2 after the
run (drop the `[0; 3]` arg; test 3/3 passes). Protocol tests 444/0 in
CI (D2 recipe regression fix holds). D3 fail (72/410 open at that tree);
D4–D6 blocked. Artifacts at ~/workspace/mcir-ci/26.2-run-43646940294/.

## Rank 2 merged to main (2026-11-15) — 08403e7f

behaviour/26.2 merged to main: rank 1 (7/7) + rank 2 (410/410),
417/651 total. CI run 43656955427 dispatched on the merged tree
(main@08403e7f); it is the first run to measure item 127's noise port.
Run 43653524277 (409/410 tree dbeb88e3) still in flight. Ranks 3–5
(234 items) continue on behaviour/26.2.

## Rank 3 started (2026-11-15) — bucket A batch 1: 9/177

Triage (same method-intersection as rank 2): A 90 / B 63 / D 24;
182 changed-named hits in B. Script + output at
~/workspace/mcir-work/26.2-behaviour/rank3-triage.{py,json}.

Batch 1 (all no-change, verified against the jars): 430 dripstone
(record→class + config threading, bodies identical), 435 submit
(vanilla CommandEncoder is GPU plumbing; Rust uses wgpu's), 452
entity_outline (GpuFormat enum churn, comment-only Rust mention), 463
BlendFunction (record re-shape, doc citations only), 468 debug_lines
(comment-only mentions), 487 loot odds (condition-form + getOptional
rename; formulas identical), 498 iceberg (record→class + context,
place body identical), 500 trunk placers (TreeFeature→TreeConfiguration
param swap, bodies identical), 502 sprite animation (RenderPearl→
Blaze3D renames; close byte-identical).

Item 451 (scoreboard/team commands) is NOT a no-change: the Rust ports
the 26.3 CommandResponseTracker form (Tracker::track/mark + send) and
26.2 uses manual one/many branching on names.size() with different
counts/returns (e.g. setScore returns value * names.size()). Deferred
to its own port task.

## Rank 3 — bucket A batch 2: 20/177 (2026-11-15)

All no-change, verified against the jars: 503 waterlogged vegetation
(config threading, bodies identical), 505 section mesh (record re-shape
+ a timing getter removed, no Rust surface), 506 cuboid item model
(quad-split hoisting; same quads, same animated condition), 507
illusioner (rangedAttackUncertainty is 14 - difficulty*4 in both — the
26.3 default method's bytecode proves it; Rust already uses it), 508
experimental redstone (entire diff is the RedStoneWireBlock rename),
509 bots chunk cache (doc comment only), 511 warden tracker command
(single-player-only Rust, tracker-vs-manual unobservable), 512 sniffer
renderer (signature re-shape), 513 narrator output (trigger removal,
already done in item 129), 514 zip pack supplier (method renames),
515 game-rules category entry (Rust already draws the 26.2 centered-text
form, no focusable widget).

## Rank 3 — bucket A batch 3: 32/177 (2026-11-15)

11 no-change + 1 port. No-change: 516 effects trigger (no trigger
logic in the Rust), 517 jigsaw (codec + canAttach, unported; Pools
bootstrap has no Rust surface), 519 gamemode switcher (all three
classes decompile identically — bytecode artifacts), 520 packs
(listResources signature re-shape), 521 sulfur cube (archetype codec
holderSet→homogeneousList; Rust loads items from tags), 522 management
(worldUpgrade* notifications are 26.3-only), 523 shapes (codecs only),
525 registries (ResourceArgument accessors exist in 26.2, absent in 26.3; Rust only cites the class), 526 command
block (codec only), 527 lightning (constants hoisted; quad bodies,
draws and loops identical), 528 drop-experience block (codec only).
PORTED: 518 /tick — 26.2's step/stopSprinting/stopStepping send
sendFailure and return (step returns 1 even on failure; stops return
0), where 26.3 throws; cmd_tick's three failure arms now use
command_failure + Ok(1)/Ok(0)/Ok(0).

## Rank 3 — bucket A batch 4: 44/177 (2026-11-15)

All no-change, verified against the jars: 529 furnace isFuel (Rust
already uses the FuelValues burn_time table from item 1), 530 leash
knot renderer (null-arg signature difference), 531 cured-zombie
trigger (predicate-type change only; same three-way conjunction),
532 spectator menu icons (identical decompiles), 534 flower block
(codec only), 535 copper chest (codec only), 536 redstone torch tick
(setBlockAndUpdate is setBlock(..., 3) in both versions), 537 tree
decorator context helpers (26.3-only; sole caller is the 26.3-only
ShelfMushroomDecorator), 538 ban notice (openUri move), 539 inventory
effects (identical decompiles), 540 test block (codec only), 541 debug
entries (identical decompiles).

## Rank 3 — bucket A batch 5: 56/177 (2026-11-15)

9 no-change + 3 ports. No-change: 543 config listener (tags() vs
getTags() rename), 545 tracked entity (identical decompile), 546 shelf
renderer (static→instance + Vec3/rotate renames, same transforms), 547
tutorial toast (identical decompile), 548 silverfish goals
(setBlockAndUpdate ≡ setBlock(..., 3)), 549 carrot block (26.2 codec
only), 550 server list entry (identical decompile), 552 fluid model
(Rust already takes materials directly, the 26.2 form), 554 chorus
plant (26.2 codec only). PORTED: 542 block-editor pause — structure
and test editors return isPauseScreen=false in 26.2 (26.3 removed the
overrides), jigsaw/test-instance keep the default true; the Editor
trait gains a defaulted is_pause_screen(), Game::pauses_game ORs the
new block_editor_pauses(). 544 /raid check — 26.2's first line keeps
a trailing space ("Found a started raid! "). 551 /unpublish — failure
is sendFailure + return 0, not a throw (same pattern as item 518).

CORRECTION (2026-11-15): diff.json's method added/removed run in the
26.2 direction — "added" means present in 26.2, absent in 26.3 (and
"removed" the reverse). Two batch-3 evidence lines had the direction
backwards (525's accessors are 26.2-only, not 26.3-only; the "codec
only" blocks carry a simpleCodec in 26.2 that 26.3 lacks). Verdicts
unchanged — no behaviour method changed in any of them — but the
evidence in rank3-progress.json is corrected.

TOOLING NOTE: never run bare `cargo fmt` (or `cargo fmt -- <paths>`)
in the 26.2 tree — this toolchain's rustfmt 1.95 reformats ~50 files
of pre-existing drift (let-else/match-arm collapsing). Format single
files with `rustfmt --edition 2024 <file>` and check git status after.

## Rank 3 — bucket A batch 6: 68/177 (2026-11-15)

All 12 no-change. 555 pale moss decorator (FEATURE→CONFIGURED_FEATURE
registry rename only), 557 snowy block / 558 slab / 559 rotated
pillar / 569 banner (26.2 codec only), 560 screenshot (RenderPearl→
Blaze3D imports), 561 social screen (toggle-close condition differs
but the Rust never ports the toggle close — Escape/Done only), 562
chat visibility (same three values; 26.3 only adds a stream codec),
563 wind-charge deflect (26.3 adds a power Vec3 to deflect; the Rust
already implements the 26.2 forms — REVERSE scales -0.5, AIM_DEFLECT
uses the unit look — and MOMENTUM_DEFLECT is unported), 564
enchantment predicate / 567 pack entry / 568 checkbox (identical
decompiles).

## Rank 3 — bucket A batch 7: 80/177 (2026-11-15)

10 no-change + 2 ports. No-change: 570 JSON-RPC logger (same log
line, builder vs direct call), 571 evoker fangs renderer
(rotateDegrees vs mulPose, GPU plumbing), 572 enchantment screen /
574 spectator team icon / 581 profile resolver (identical
decompiles), 573 criterion validate (predicate-type re-shape;
validate unported), 575 edit-world screen (Blaze3D.openPath vs
Util.getPlatform().openPath), 577 allay vibration (pos.immutable()
vs new BlockPos), 579 turtle egg goal (setBlockAndUpdate ≡
setBlock(..., 3)), 580 wood type (26.3 adds POPLAR, but the Rust has
no WoodType enumeration — comments only). PORTED: 578 armadillo
ball-up — 26.2's checkExtraStartConditions is onGround() only, 26.3
added !isInLiquid(); dropped the liquid check and the dead helper.
582 management schema — 26.3's four world/upgrade_* notifications
are absent in 26.2; removed from METHODS, test count 89 -> 85.

## Rank 3 — bucket A batch 8: 89/177 (2026-11-15)

All 9 no-change — bucket A is now complete except deferred item 451
(scoreboard/team commands, its own task). 583 biome tags (26.3 adds
18 HAS_ABANDONED_CAMP_* tags; the Rust only uses the is_* category
tags, identical in both versions), 584 growing plant (26.2 codec
only), 586 end city / 587 mansion (26.3 adds a biome pre-check to
findGenerationPoint; the Rust has no per-structure generation-point
port), 588 ocean monument (biomeResolver vs biomeSource+sampler —
same query), 589 jungle temple (RedStoneWireBlock rename), 591 tab
bar / 592 system toast (identical decompiles), 594 recipe clearGrid
(26.3-only Prediction param; the Rust models no prediction).

## Rank 3 — item 451: 90/177 (2026-11-15)

Bucket A is now complete (90/90). Item 451, scoreboard/team
commands, was the deferred real port: 26.3 routes the one/many
feedback through CommandResponseTracker, which counts only nonzero
tracks; 26.2 branches manually on names.size() and counts every
target. Ported: set (branch on the target count — a set to 0 with
one target is a single success), enable (message by names.size(),
return the unlocked count), team join/leave (count every member;
a teamless member still succeeds and counts). add/remove/reset/
display/numberformat/operation were verified equivalent and are
unchanged. New regression test pins the three divergences; all 9
scoreboard_cmd tests pass.

## Rank 3 — bucket B batch 1 (part 1): 92/177 (2026-11-15)

Two ports. 437 /effect + /enchant: same Tracker→manual family as
item 451 — message choice and multiple-count now use the target
count; returns stay the success counts; the duration computation
was already the 26.2 inline form. 423 sculk: canPlaceGrowth now
checks sensor/shrieker blocks explicitly (the
sculk_growth_inhibitors tag is 26.3-only — verified absent from the
26.2 jar); the worldgen cursor now does 26.2's post-move 15-block
spread check with no stuck-kill (26.3 pre-checked 12 blocks and
killed stuck cursors). Sculk tests 3/3 pass.

## Rank 3 — bucket B batch 1 (part 2): 94/177 (2026-11-15)

424 /test: no-change — the Rust is already the 26.2 single-level
form; 26.3's cross-dimension test lookup is unported. 429 /time:
ported — 26.3's five "already" errors (already-at-time,
already-at-marker, already-paused/running, already-same-rate) do
not exist in 26.2; all four checks removed from the /time arms.

## Rank 3 — bucket B batch 1 (part 3): 96/177 (2026-11-15)

422 social: no-change — the diff is the authlib package move plus
26.3's postPresence try/catch wrap; the Rust's friends service is a
local stub with no real network call. 425 fortress: no-change —
piece methods differ only by the Builder vs Accessor parameter
type, and FORTRESS_ENEMIES carries the same values in IntProvider
vs (min,max) form; the Rust has no enemies-list surface.

## Rank 3 — bucket B batch 1 (part 4): 97/177 (2026-11-15)

431 advancements: ported — 26.2's advancements key always closes
to the game (26.3 restores the last screen); added
close_advancements_to_game and routed KeyL to it. Tab selection
(left-click), DisplayInfo-derived tab icon/title, and onClose are
already the 26.2 forms; AdvancementWidget is identical.

## Rank 3 — bucket B batch 1 (part 5): 99/177 (2026-11-15)

427 world options: ACCEPTED GAP — 26.3 merged LAN publishing into
WorldOptionsScreen; 26.2 keeps WorldOptionsScreen (immediate-apply)
and the 26.2-only MultiplayerOptionsScreen separate. The Rust ports
the 26.3 merged screen; the pause menu already wires the 26.2
Multiplayer Options button to it, and every underlying operation
matches 26.2. Only the layout differs; splitting the 1,902-line
screen is deferred to a later UI pass (in-code note,
screens.rs:455-458). 428 options screens: no-change — all 12
classes decompile identically.

## Rank 3 — bucket B batch 1 (complete): 102/177 (2026-11-15)

434 GUI render: no-change — PiP differs only in render-pass
plumbing, the rest identical. 436 title/multiplayer: ACCEPTED GAP
— the only real divergence is editServerCallback (26.2 keeps the
old ping state after an edit, 26.3 resets to INITIAL; the Rust
re-pings, matching 26.3); porting it needs the Edit variant to
carry the list back, deferred. Everything else in the item is
identical or the established openUri pattern. 438 screen effects:
no-change — the item-activation math is identical in both
versions; submit() differs only by render-state vs direct reads.

## Rank 3 — bucket B batch 2 (part 1): 104/177 (2026-11-15)

439 /place feature: ported — 26.2 takes a configured-feature
registry key only; the 26.3 inline-feature form (tree arg,
dispatch branch, place_inline_feature/configured_inline/
decode_inline, InlineFeatureCheck, codec arm, both test files
and both harnesses) is removed. 440 mineshaft: no-change — the
Builder vs Accessor parameter rename only, as with the fortress.

## Rank 3 — bucket B batch 2 (part 2): 108/177 (2026-11-15)

441 beehive, 442 GUI renderer, 443 spike/fill-layer, 444 brush:
all no-change. 441 is the server-signature re-shape + an unused
loot pos arg + the unported Prediction flag. 442's UBO order
differs but the Rust's uniform layout is self-consistent with
its own shaders. 443 is the record vs class re-shape. 444 is
setBlockAndUpdate vs setBlock(...,3) + rotateDegrees vs mulPose.

## Rank 3 — bucket B batch 2 (part 3): 109/177 (2026-11-15)

445 gamemode + debug: ported — /gamemode and the debug key set
the world's default game type for the singleplayer owner (26.2
only); FPS line loses the 26.3 @Hz suffix; the 26.3-only Terrain
Rendering and Window debug lines are gone; chart labels back to
FPS / FPS + TPS; the 26.3-only debugGuiScale option is removed
(overlay always at GUI scale). Also removed the now-dead
TimeMarker::occurs_at (item 429's leftover).

## Rank 3 — bucket B batch 2 (complete): 112/177 (2026-11-15)

446 dispenser: no-change — the honeycomb-wax and mud-tag forms
are already 26.2 in the Rust; the rest is setBlockAndUpdate vs
setBlock(...,3) or identical. 447 DeviceInfo: no-change —
identical apart from the package move. 448 item renderer:
no-change — the glint math is identical; the pass-structure diff
is unobservable in the Rust's shader glint, and the shade
override is None on 26.2 assets.

## CI run 43653524277 (2026-11-15, tree dbeb88e3, 409/410)

Profile 5/0/12 — unchanged (first_person, food, lan, light,
resource_pack). D1 pass. D2 fail: cargo check 2 errors, the
rule_test_bounds E0061 — the fix (b232b15b) landed after this
tree, so the next run measures it. D3 fail: 1 rank-2 item open
(127, since closed and merged). Run 43656955427 (merged tree
08403e7f, first run with item 127's noise port) still in flight.

## Rank 3 — bucket B batch 3 (part 1): 116/177 (2026-11-15)

449 enchant effects, 450 books, 453 streaming, 454 igloo/
shipwreck: all no-change. 449's impulse math is the same
rotation two ways; 450's book clicks are already left-click;
453's compile-start clock is Horizon's own; 454 is the
Builder/Accessor rename.

## Rank 3 — bucket B batch 3 (part 2): 120/177 (2026-11-15)

455 piercing weapon, 456 multiline edit, 457 data commands,
458 title/NBT: all no-change. 455's reset happens in both
versions via Player.onAttack; 457's compute subcommand is
26.3-only and unported (the /compute command itself is item
510); 458's Tracker form tracks every target, so it equals
26.2's manual branching.

## Rank 3 — bucket B batch 4: 128/177 (2026-11-15)

459 gizmos, 460 ore, 461 bed rule, 462 monster room, 464
bonus chest, 465 test data, 466 sapling, 467 render section:
all no-change. 461 and 466 were already ported in earlier
items (bed_rule_accepts_the_26_2_explodes_field,
lone_dark_oak_sapling). 459's translucent lines are
no-depth-write in both versions; 467's fade math is
identical, field vs parameter only.

## Rank 3 — bucket B batch 5: 136/177 (2026-11-15)

469 ruined portal, 471 cube map, 472 stats screen, 473
biome builder, 474 ocean ruin, 475 guardian, 476 debug
charts, 477 transfer. PORTED: 469 (26.2's ruined-portal
drip columns place netherrack/magma unconditionally; 26.3
checks replaceability inside placeNetherrackOrMagma) and
476 (chart gridline labels back to 26.2's uppercase FPS/
TPS — item 445 had only fixed the titles). The rest are
no-change: 473's biome-table and spawn-target differences
are data-driven.

## Rank 3 — bucket B batch 6: 144/177 (2026-11-15)

478 wandering trader, 479 container events, 480 pumpkin,
482 creaking heart, 483 dragon egg, 484 nether fossil, 486
locator bar, 488 command minecart. PORTED: 483 (the Rust's
dragon-egg teleport was missing the world-border check
both vanilla versions have). 479's drag button is already
left-click in the Rust; 483's 2015-vs-particles difference
was already ported (2015 is the sim's internal channel).

## Rank 3 — bucket B batch 7 (COMPLETE): 154/177 (2026-11-15)

489 panda, 490 phantom, 491 trade goal, 492 file
download, 493 narration, 494 toast, 495 argument types,
496 bucket, 497 side chain, 499 template piece. Bucket B
is now CLOSED (63/63). PORTED: 495 (the five 26.3-only
argument types removed from the id table — verified
against the 26.2 registry data, 57 entries). 491 and 493
were already ported in earlier items. Remaining rank 3:
bucket D, 23 items.

## Rank 3 — bucket D batch 1 (part 1): 155/177 (2026-11-15)

470 CommandResponseTracker: the class is 26.3-only; the
Rust's Tracker helper was audited use by use. PORTED: /tag
add/remove/list (26.2 branches on the target count, not
the changed count). Equivalent as-is: scoreboard,
/forceload (26.2 branches on changed chunks), /loot,
/swing, /playsound. The /item sites belong to item 419.

## Rank 3 — bucket D batch 1 (part 2): 156/177 (2026-11-15)

419 /item command: the big one. 26.2's /item is the SPLIT
form (setBlockItem/setEntityItem/modifyBlockItem/
modifyEntityItem, one SlotArgument, replace+modify only);
26.3 unified it behind ItemAccessor + SlotSourceArgument
with fill/override. The tree, cmd_item, cmd_item_modify
and the execute `if items` condition are now the 26.2
forms (single slots, 26.2 error/message keys, slot ints
from a new args::slot_id mirroring 26.2 SlotRanges);
`if slots` is gone (26.3-only). The whole 26.3 command
slot-source apparatus is removed (ArgType::SlotSource,
the inline/contents machinery, the datapack slot_sources
store, the item-contents corpus, the structured-item-slots
exec group, the slot-range-codec harness). The codec's
slot_source decode arms STAY: minecraft:slot_source_type
exists in 26.2 — only the minecraft:slot_source registry
and the command argument type are 26.3-only.

REPORT HOLE (found during 419): ExecuteCommand is absent
from diff-26.3-26.2/report.json although it genuinely
differs (118 methods in 26.3 vs 116 in 26.2; javap shows
26.3 referencing SlotSourceArgument 14 times where 26.2
references SlotsArgument 6 times). The slot-condition
part was ported here via SlotSourceArgument, but the
rest of ExecuteCommand's 26.2 differences (notably the
data-condition ArgProvider/DataAccessor re-shape) are
covered by NO work item and invisible to D3.

## Rank 3 — bucket D batch 2: 157/177 (2026-11-15)

418 Realms screens: the big one. 26.2's Realms has no join
codes, no product page, no tier configuration and no invite
codes — all four 26.3 surfaces are removed (JoinCode screen
+ header Join button, the whole PDP, the settings tab's
distance rows + ScrollableLayout scroll view, the
InviteCodes/EditCode screens + the Players tab's button).
PORTED: AddRealmPopupScreen (26.2-only, 320x172 popup with
Buy/Trial/close) + the trial diamond over the Add Realm
button; saveSettings unconditional (26.2 has no change
check — 26.3 skips the call when nothing differs), with
blank-to-empty name/description semantics; RealmsRegion
back to 26.2's 23 (CanadaCentral, MexicoCentral,
SouthAfricaNorth, WestUs3 are 26.3-only); CreateRealm
passes the name/description raw (26.3 trims); region
preference Done applies only when a region is picked
(26.2's null-guard). Verified no-change: ConfirmLinkScreen
+ extendRealms, the template screen (parse already
normalizes "" to None), the news/notification classes
(URI-to-String re-shapes), the terms/template clicks, the
subscription tab, RealmsUtil, and all extract* render-state
classes. KNOWN GAP: the popup's carousel pictures — 26.2's
client.jar ships no realms namespace, so vanilla draws no
carousel picture in a standard install either; a resource
pack adding assets/realms/textures/gui/images/*.png would
show pictures in vanilla but not here. The client.rs tier
types + invite-code methods stay for item 421. Two test
failures pre-date this item (client cookie, optimization
stamp — both fail at the pre-418 base too).

## Rank 3 — bucket D batch 3: 158/177 (2026-11-15)

421 Realms client: the invite-code service is gone
(frontendV2Url, the frontend_url field, the x-mojangauth
header, all five invite-code methods, the InviteCode
DTOs), and so is the tier configuration (the three
distance endpoints, both DTOs). PORTED: the Client-Ref
header is gone — 26.3's addVersionHeaders adds it, 26.2's
sends only X-NetworkProtocolVersion + Is-Prerelease;
WorldTemplate link/trailer are plain Strings again
(26.2's parse uses getStringOr with "" default; 26.3 made
them nullable URIs), so the template screen checks
emptiness instead of Some. Verified no-change:
RealmsClient.execute itself, and the notification
UrlButton (already a plain String). path_segment stays —
the invites, template and minigame calls still use it.

## Rank 3 — bucket D batch 4: 159/177 (2026-11-15)

420 Post chains: two std140 ports. 26.2's Std140Builder
pads a vec3/ivec3 to a full 16-byte slot (26.3 packs it
as 12), so pack_block now honours the size and a vec3
followed by a float lands the float at word 4. And
GlobalSettingsUniform: 26.2's Globals block is
ivec3/vec3/vec2/float/float/int/int at 56 bytes, 26.3's
is ivec3/float/vec3/float/vec2 at 48 — correcting item
442's no-change, the layout IS observable, because
translated pack shaders declare the block themselves
and naga lays it out from the declaration; the old
48-byte block misread GlintAlpha, GameTime, the blur
radius and UseRgss in 26.2 packs. Globals.bytes() is
now the 56-byte 26.2 layout, with the core_lines /
core_sky ripple (signatures, block sizes, fixtures).
Verified no-change: PostChain, PostPass, SamplerCache
(renderpearl-to-blaze3d moves and renames), the
ShaderManager async rework (neither form is modelled;
#include preprocessing is post_glsl's), and the
LevelTargetBundle OIT-vs-sorting targets (chains see
only minecraft:main in both versions).

## Rank 3 — bucket D batch 5: 160/177 (2026-11-15)

426 Transient blocks: already the 26.2 form — the packet
is absent from the 26.2 jar and the Rust keeps the
queue/drain/render path inert. Verified no-change: the
occlusion graph's transient-removal call (inert with the
module) and its empty-section bookkeeping (unmodelled),
and LevelExtractor (the drain, extractPlayerState and
the chunk fade-in option are all 26.3-only or
unmodelled; the outline condition is the same from a
different source).

## Rank 3 — bucket D batch 6: 161/177 (2026-11-15)

432 Animate-tick blocks: the poplar trees are 26.3-only
(0 poplar fields in 26.2's Blocks, 23 in 26.3's), so the
poplar leaves entries, the Leaves poplar flag, the
ambient leaves sound and its log table are gone — 26.2
has no ambient leaves sound at all. Verified no-change:
the falling-leaves particles (26.2 has them inline in
LeavesBlock; the per-type chances match 26.2's Blocks
registration exactly), the bonemeal methods (bodies
identical, only 26.3's BonemealSource parameter
differs), the onLand/onPlace/useWithoutItem calls
(setBlockAndUpdate vs setBlock(...,3)), the repeater
wire-connection rule (the Rust already is 26.2's static
form), and the per-block codecs (26.2 has them, 26.3
removed them — unmodelled).

## Rank 3 — bucket D batch 7: 162/177 (2026-11-15)

433 Codecs: RegistryFileCodec's unknown-element error
is 26.2's bare form ("Failed to get element {id}";
26.3 appends " from registry ...") — fixed in codec.rs
and item.rs, with the inline-loot test's expectation.
RangeSlotSource requires `source` in 26.2 (an entity or
block-entity target; "container" is not one — 26.3 made
the field optional with a container default), so the
slot_range check now requires and validates it. Verified
no-change: the loot-condition combinators (the Rust
already requires inline lists/direct terms, which is
26.2's DIRECT_CODEC form), the holder-set homogeneity
check (the Rust does not check holder lists — a
documented partial-check gap), and the builder/entry
re-shapes.

## Rank 3 — bucket D batch 8: 163/177 (2026-11-15)

481 Frog lay-spawn: 26.2 uses TryFindLandNearWater,
26.3 renamed it TryFindLandNearLiquid with a fluid-tag
parameter (whose tag holds only water). PORTED: the land
search's water test is now a block test — 26.2 checks
the block below the neighbour is a water block, while
26.3's fluid-state test also accepts waterlogged blocks.
Verified no-change: the Manhattan iteration order (26.2's
withinManhattan iterator is exactly the Rust's
manhattan_box), the guard, the cooldown, and the
lay-spawn behaviour itself.

## Rank 3 — bucket D batch 9: 164/177 (2026-11-15)

485 Armour trims: already the 26.2 form — the Rust
implements 26.2's atlas mechanism at runtime (the
material format with asset_name + override_armor_assets,
PalettedPermutations' mapping keyed on trim_palette, no
glint on the trim layer), where 26.3 moved to runtime
PalettedTextureManager textures with palette_id
materials. Only the attributions needed fixing
(TrimTextureKey/PalettedTextureManager to the 26.2
names). Verified no-change: TrimPredicate (the trim
predicate is accepted unchecked, so its holder-set
difference is unobservable).

## Rank 3 — bucket D batch 10: 165/177 (2026-11-15)

510 /compute is 26.3-only (ComputeCommand,
LootContextSources and both context providers are
absent from the 26.2 jars, and 26.2's DataCommands has
no compute source): the whole implementation, the tree
registration, the /data modify compute source, the
context_sources helper and the exec corpus group are
removed. The bare 'compute' parse-corpus entry stays —
it now measures the unknown-command error on both sides.

501 ResourceOrIdArgument: the feature and
context-provider argument types are 26.3-only; the
dead context-provider decode arms, provider_fields
and fixture mappings are removed (the /compute tree
nodes went in item 510). LootModifierArgument's
ROOT_CODEC vs DIRECT_CODEC decode identically
(DFU withAlternative is either + xmap unwrap), so
the Rust's either-errors stand; /place feature
already uses the 26.2 ResKey form.

504 ContentsSlotSource: 26.2 has no
minecraft:slot_source registry and no
SlotSourceArgument — SlotSources.CODEC is
withAlternative(typed, inline-list), not a holder
set. The codec's holder-set helper is replaced by
a withAlternative helper (no #tag, 2-part either
error) for the slot_source and item_modifier arms
alike; the three slot_source tests are re-pinned to
the 2-part errors. The removed classes
(ArrayItemProvider, GrowableMutableContainer,
ComponentSlotCollection) have no Rust references.

524 EmissiveRisingParticle is a pure rename of
26.2's SoulParticle (behaviour-identical classes,
same provider registration): the Rust already
implements the 26.2 form; only the two comments
were re-attributed.

533 ArgProvider is 26.3's rename of 26.2's
DataProvider (same tree, same wraps; the only
other DataCommands delta is the compute source,
removed in item 510): only the tree_data.rs
comment was re-attributed.

553 NarrationTrigger is 26.3-only (26.2's
scheduleNarration takes only delay + suppression):
the narrator crate is already the 26.2 form, no
change.

556 The whole blockscan package is 26.3-only;
26.2's SculkBlock.canPlaceGrowth is a direct
betweenClosed loop counting sensor/shrieker,
which the Rust already implements exactly — only
the comment was re-attributed.

565 DensityFunction$Axes is a 26.3-only empty
marker annotation (CLASS retention, no members,
no runtime behaviour): nothing to port.

576 ItemActivation is 26.3's extraction of
26.2's ScreenEffectRenderer fields; the behaviour
is identical, and the Rust implements it exactly
— only the doc attributions were fixed to the
26.2 names.

585 SpawnTargetPoint (26.3's NoiseSpawnFinder
record) has no 26.2 equivalent; the Rust already
implements 26.2's Climate.findSpawnPosition over
ParameterPoint targets.

Pre-existing horizon-world test failures found
while running the full suite (not regressions
from this session): the 18 growth_tests fail at
fixture setup because the prepared bundle's
carvers.json is empty ({} since Nov 11);
doors_beds_and_plants_place_both_halves fails on
the bed head half (test untouched since
publication). The aquifer cache test's 26.3-only
gradient fixture is fixed in 63afd0b6.

590 SteppedInterpolationHandler is 26.3-only
(26.2 has a single InterpolationHandler, which
the Rust already implements): only the six
comments naming 26.3's LinearInterpolationHandler
were fixed.

593 /test locate was the 26.3 form (cross-
dimension search, 3 out, /execute in tp,
dimension suffixes, outputPlayerCoordinates):
ported to 26.2's same-level search, 2 out, plain
/tp and numeric distance. TestCommand$Info is
26.3's playerAndTestInfo bundle; 26.2's
createTestPositionAround is inline.

566 OIT is entirely 26.3-only: the 26.2 client
jar has no net/minecraft/client/renderer/oit/
package at all (26.3 has OitPipelineSet,
OitRenderPassProvider, OitStage), so the Rust
OIT implementation came out in four parts:
(1) game-side OIT tagging dropped; (2) the
render pipeline deleted — gfx/oit.rs (997
lines), gfx/oit_features.rs (275 lines) and
five oit*.wgsl shaders, plus call-site cleanup
(25 files, +81/−2539); (3) the F3+X improved-
transparency debug key removed — it is 26.3-
only (no key.debug.improvedTransparency in
26.2's lang, zero references in 26.2's
KeyboardHandler), while the improvedTransparency
graphics option itself EXISTS in 26.2 and stays;
(4) warning cleanup to D2's zero-warning bar
(dead weather pipeline/draw_with, one pre-
existing unused mut, one unused doc comment).
Render lib suite: 2045 passed, 1 failed, 66
ignored (Modrinth test skipped); the failure is
the pre-existing animate atlas-size assertion,
unrelated to OIT.

RANK 3 CLOSED 177/177 — see rank3-progress.json
for per-item evidence.

## CI run 43688968589 dispatched on main (2026-11-15)

Rank 3 merged to main as c8c8e835 (rank 3 177/177; this merge
carries the D2 fix b232b15b for the rule_test_bounds E0061 errors
plus the item-566 warning cleanup to main). Run 43688968589
dispatched on main@c8c8e835, in progress — first run to measure
the rank-3-closed tree; expect D2 to clear (E0061 fix + 0
warnings). Porting continues (ranks 4–5) while CI runs.

## CI run 43688968589 completed (2026-11-15)

Run 43688968589 on main@c8c8e835 completed in 1h30m, conclusion
success. Profile 5/0/12, unchanged (first_person, food, lan,
light, resource_pack pass). D1 PASS, D2 PASS, D3 PASS — the first
run with all three passing together: D2 cleared as expected (the
E0061 fix and the 0-warning item-566 cleanup are on main).
D4/D5/D6 blocked (D4/D5 on the known staged-candidate profile
stamp; D6 needs the human playtest). Verdict: blocked, verified
false. Artifacts at ~/workspace/mcir-ci/26.2-run-43688968589/.

Rank 4 CLOSED 55/55 (all no-change). The rank-4
items are packages of classes new in 26.2 (the
26.3 side dropped or renamed them); each was
triaged by name-intersection against the Rust
tree and the Java oracles: 22 with zero
references anywhere, 1 oracle-only (639,
RegistryOps$RegistryInfo, named only by
VanillaSimpleBlockOracle — correct, oracles run
against the 26.2 jars), and 32 with Rust
references, every one classified: comments
citing a 26.2 class that exists in 26.2
(AxeItem/HoeItem/ShovelItem tables, PotionBrewing,
ConfiguredFeature, ItemInHandRenderer,
GlslPreprocessor, ClockInstance, TrimSpriteKey
and the rest), implementations already ported to
26.2 (density GraphNode variants, Direction8
top-level enum, synth.rs from item 127, brewing's
65 mixes, SpawnFinder, both DirtPathBlock and
RedStoneWireBlock spellings in classes.rs), or
name collisions (Sum, IntRange, Mask). No code
fix anywhere. See rank4-progress.json.

RANK 5 CLOSED 2/2 — ALL 651 ITEMS CLOSED (2026-11-15). See
rank5-progress.json for per-item evidence.

Item 650 (unported, 1135 classes) PORTED. Three real ports plus a
systematic decode fix:

- Feature-decode config lift (78cfb3c0): 26.2 wraps configured-feature
  parameters in `config`; the Rust decoded only the flat form, so every
  real 26.2 configured feature failed to decode. Fixed with
  worldgen::lift_config at FeatureDefinition::decode + FeatureCatalog
  expansion (signal-checked tests).
- TagEntry `name` form (78cfb3c0): 26.2 names a tag entry's tag in
  `name`; 26.3 moved it to `items`. The Rust read only `items`, so 26.2
  tag entries dropped nothing. Fixed with LootData::tag_entry_items
  (signal-checked test).
- Ten missing 26.2 features (37fca9be): kelp, seagrass, sea pickle
  (worldgen/aquatic.rs); basalt columns, basalt pillar, glowstone,
  nether forest vegetation, twisting vines, weeping vines
  (worldgen/nether.rs); desert well (worldgen/desert_well.rs). All 10
  classes are added in 26.2 and had no Rust decode arm. Each place()
  mirrors its decompiled 26.2 feature, including random-draw order and
  vanilla quirks; desert well needed a new PlacementContext method
  (set_block_entity_loot_table_seeded) because vanilla seeds the sus
  sand with pos.asLong() and draws nothing for it. All 10 parity
  harnesses pass against the 26.2 oracle on BOTH jars — 769 fixtures,
  1,617,895 output lines matched (kelp 124/5,768; seagrass 169/3,056;
  sea pickle 96/6,035; basalt columns 64/384,404; basalt pillar
  36/14,508; glowstone 40/501,662; nether forest vegetation 88/90,711;
  twisting vines 68/492,191; weeping vines 36/86,192; desert well
  48/33,368). Three basalt-columns fixes came out of the harness:
  Feature.setBlock writes with flag 3 (a861f950), betweenClosed walks
  x fastest (90d21d13), and the lava-ocean test reads the chunk
  generator's sea level, not the world's (db9385fc).
- State-provider long-name aliases (145d4f63): 26.3 shortened the
  provider type names; 26.2 data uses the long forms (296 uses across
  8 types) and decode_typed accepted only the short names, so every
  26.2 configured feature using a provider failed to decode. Long names
  are now aliases (fields verified identical for all 8 types).

Harness infrastructure fixed on the way: the oracle's
block-registry.nbt defaults were keyed by the encoded `id` field
(empty in 26.2 — every default collapsed onto one key), now keyed by
the block registry key (VanillaCollisionOracle, VanillaWorldOracle);
the simple-block harness bundle/catalog ported to 26.2 provider forms
+ the wrapped config form + a count-0 horizon:empty (26.3-only
RandomBlockProvider has no 26.2 equivalent); the oracle's
getBlockEntity handler answers the typed overload with an entity of
the requested type (brushables map + reflective loot reads).

Remainder of 650 classified no-change (see rank5-progress.json for the
full per-class notes): the 30 removed loot number providers, the
~25 feature classes (whose live-form gap was the config lift),
BlockPredicate components (no vanilla 26.2 advancement uses it),
JumpOnBed / SleepInBed / Digging brains (SleepInBed's conditional
last_slept set is behaviorally identical to 26.2's unconditional one:
26.2 startSleeping cannot fail and the Rust's false branch is
unreachable at its single caller), ExecuteCommand countItems (already
ported as condition_items), RootPlacer + NoiseProvider decode shapes
(match the 26.2 JSON), and the enchantment ReplaceBlock effect (no
Rust implementation in either version, no 26.2 vanilla enchantment
uses it — a documented gap in the base, not a divergence).

Item 651 (new-unported, 75 classes) NO-CHANGE: carver config classes
already implemented, MapColor a name collision, 2 oracle-only, 65
zero-hit.

Rank-4 item 596's ledger entry was amended: 10 of its 17 classes were
genuinely unimplemented at rank-4 close and are now ported under item
650 (see rank4-progress.json).

Known gaps left open (not rank items): the simple-block harness's own
corpus still uses random_block providers directly in definitions and
mode-"map" cases parse via configuredCodec (which needs a "config"
key) — both are pre-existing 26.2 gaps in that harness, documented,
not addressed here. The unknown launch crash blocking
command_exec/save_roundtrip is still undiagnosed (second track).

## CI run 43704536595 dispatched on main (2026-11-15)

Rank 5 merged to main as f56493a6 (all 651 behaviour items closed).
Run 43704536595 dispatched on main@f56493a6, in progress — first run
to measure the fully-closed tree (rank-5 ports: config lift, TagEntry
name form, 10 missing features, provider long-name aliases). Porting
is complete; remaining work is the second track (unknown launch crash
blocking command_exec/save_roundtrip) and the D6 human playtest.

## Cushion fix merged to main (2026-11-15, 5672c415)

The command_exec fixture-control abort (run 43688968589: the harness
aborted at a Java fixture-control prerequisite because
vehicle_boolean_corpus.py summoned minecraft:cushion with
expect_result=1, and cushion is 26.3-only) is fixed on main: the
cushion kind is removed from vehicle_boolean_corpus.py (4cb9dcad).
Branch behaviour/26.2 was synced with main first (d9f6e62d, picking up
f45667f7's harness-subset.txt + status.json fix, plus the patch-0005
gate changes the branch had never merged), then merged to main as
5672c415. The next CI run on main should show command_exec getting
past fixture control; save_roundtrip's separate launch exit is still
being diagnosed locally (second track).

## Launch crash DIAGNOSED + FIXED (2026-11-15, second track)

The "unknown launch crash" blocking command_exec/save_roundtrip was NOT
a port bug. It was two missing system libraries, found by running
save_roundtrip locally and reading the game's own control.log
(/tmp/hzrtdgin4f8h/control.log, then /tmp/hzrtps1a_c32/control.log):

1. `libxkbcommon-x11.so could not be loaded` — the game panicked in
   xkbcommon-dl at startup (winit X11 keyboard init). The VM/runner had
   libxkbcommon but not its X11 extension. Fixed locally with
   libxkbcommon-x11-0 1.6.0-1build1 (+ its dep libxcb-xkb1 1.15).
   (Note: apt-get update on this VM hangs on the cogentco mirror in
   sources.list; the .debs were fetched directly from archive.ubuntu.com
   and installed with dpkg -i.)
2. `No Vulkan or OpenGL graphics driver found` — the runner/VM has the
   Vulkan loader (libvulkan1) but no Vulkan driver, and no EGL for the
   OpenGL fallback. Fixed locally with mesa-vulkan-drivers
   25.2.8-0ubuntu0.24.04.4 (lavapipe, the software Vulkan driver).

With both installed, the 26.2 client launches under Xvfb and renders on
llvmpipe (Vulkan, "llvmpipe Mesa 25.2.8 (LLVM 20.1.2)", ~0.84 fps
software) — wait-ready succeeds, 67 chunks, 45 mobs, player joined.

CI fix: port-verify v6.7 (358736da) installs libxkbcommon-x11-0 and
mesa-vulkan-drivers in the system-libraries step. Run 43704536595 (in
flight, measures f56493a6 pre-v6.7) will still fail save_roundtrip at
launch; the NEXT run on main is the CI verification of this fix.

Side finding — the vanilla_login bot: it fails to log in during the
harness's world build (TimeoutError locally once, EOFError on CI and in
the re-runs) but logs in instantly against an idle 26.2 server (traced
2026-11-15: SetCompression + LoginSuccess in 0.0s). The bot code and
protocol (776) are correct; the loaded server (4096-chunk forceload +
scene commands on a small box) does not answer the handshake within the
bot's 30s window, and the server shutdown then closes the connection.
Consequence: the RoundTrip player never joins, so no player .dat is
written and the player-data roundtrip is silently unexercised. A
harness-load artefact, not a port bug; the harness proceeds regardless.

## save_roundtrip now MEASURES (local, 2026-11-15)

First full local run with the libraries installed: the game launches,
saves, and the harness compares — `save round-trip parity FAILED (2962
unexplained differences)`. The launch is fixed; these are real save-path
gaps, the next work item:

- ~2402 "Not a map" chunk-section errors: vanilla reloading Horizon's
  save reports block states as bare names ("minecraft:air") where it
  expects {Name, Properties} maps; the NBT diff shows the same loss the
  other way (vanilla "256 differ, first #768
  minecraft:grass_block[snowy=false]" vs horizon "minecraft:grass_block").
  Horizon's chunk-section palette writer drops state properties and the
  map form.
- 79 VANILLA VIEW block-entity diffs: Horizon's resave writes no block
  entities ("The target block is not a block entity" for jigsaw,
  structure_block, sculk_sensor, beehive, spawner, command blocks,
  beacon, signs, ...). Block-entity data is lost on save.
- Remainder (~480): heightmap/section diffs on chunks vanilla left
  blank (Horizon finishes them), plus ordinary simulation drift.
  (The 4 pre-known vanilla scene-command errors — decorated_pot sherds,
  BlockState "No key Name" — persist, unchanged.)

Run logs: /tmp/save-roundtrip-local3.log (full), control.log at
/tmp/hzrt1ylm0m8y/control.log. Run 3 took 3m38s with the build cached.

## save_roundtrip gaps: diagnosis corrected + palette fix (2026-11-15)

Decomposing the 2962 by stage, the earlier "resave writes no block
entities" reading was wrong. The counts: 321 NBT-compare diffs, 79
VANILLA VIEW diffs, 2562 vanilla-log problem lines (2400 "Recoverable
errors when loading section" + 79 "Failed to create block entity" + 77
IllegalStateException + 2 AirBlock/ClassCastException + 4 EntityStorage
serialization errors). Horizon DOES write block entities — vanilla tries
to create them when loading H — but the blocks at those positions are
air in H's save because the palette "Not a map" errors made vanilla
fall back to defaults. The stage-3 NBT compare shows NO block-entity
diffs (only ignored timer diffs): Horizon's written block-entity NBT
already matches vanilla's exactly. The 79 VANILLA VIEW diffs and the
"Failed to create block entity" errors are all downstream of the
palette bug.

Root cause — a 26.3-only codec change the save path never ported:
26.2's BlockState.CODEC is the map form ({Name, Properties}) for every
state (BlockState.java:10: `codec(...).stable()`). 26.3 added the
string-or-map codec (CONSTANT_OR_DISPATCH_CODEC: default states as a
bare block name). The Rust save path implemented the 26.3 codec in two
places, and vanilla 26.2 rejects bare strings in palettes ("Not a
map"). H's palettes held 7,789 bare-string entries vs 14,782 maps;
V0's held 22,136 maps, 0 strings.

Fix (batch 1, Rust): palettes and entity block states now always write
the map form. `save.rs` encode_with: palette entries are always
`s.to_nbt()` (None arm: the air map); the now-dead
`SaveTables.default_state` field is removed. `entity_nbt.rs`
state_to_nbt: same fix, comment corrected (the old comment claimed the
26.3 codec and the 26.3 `{id, properties}` form). `game/tnt.rs`
block_state_nbt: the None arm resolves the TNT default state through
the registry instead of a bare string. `block_entity_nbt.rs`: the
piston blockState fallback is the air map. The read path
(state_from_nbt, BlockState::from_nbt) still accepts both forms — only
the write path changed. Biome palettes stay bare strings: vanilla's
biome palette IS a list of registry-name strings.

Harness fixes (batch 2, surgical, in test_save_roundtrip_parity.py):
- Heightmaps on promoted chunks: the 44 `carvers`-status chunks have
  no heightmaps in v0 (Horizon finishing them computes the four
  regular ones), so the diff is presence-only, not a value
  disagreement. The promoted relabeling now also relabels
  `chunk.Heightmaps.<name>` paths, but ONLY when the vanilla value is
  None; value diffs still fail. The IGNORED promoted pattern widens
  from the two `_WG` heightmaps to `Heightmaps.\w+`. Verified offline
  against the old run's saved worlds: heightmap diffs 176 -> 0
  unexplained (189x4 now ignored under chunk.promoted, the 145 blank
  ones included).
- DespawnDelay: added to the entity ignore pattern ("entity simulation
  advances"). Proven exact: v0 level.dat Time=842, H Time=979 — 137
  ticks, exactly the DespawnDelay drift (47378-47241=137).

Offline re-compare of the old run's worlds with the harness fixes:
unexplained NBT diffs 321 -> 144 (141 blocks + falling_block
BlockState + block_display block_state + version_history), all
Rust-side and all addressed by batch 1 except version_history (next
batch). The full harness re-run after the rebuild is the real
verification.

Unit-test fallout of the codec fix (all in the 26.3 codec's shape):
six tests pinned the bare-string form and were updated to the map form
— falling_decode_absent_state_saves_actual_sand_without_raw,
falling_decode_modern_boundary_keeps_shared_legacy_reader (which also
exposed a pre-existing wrong expectation: its {Name: gravel} case
expected sand, but the shared reader decodes it as gravel, per vanilla's
BlockState.CODEC — fixed to gravel), the enderman carriedBlockState
assertion (grass_block's default state carries Properties
{snowy: false}, which the map form now shows), the piston blockState
fallback, and the two TNT block_state tests (saved_block_state... also
had its unstable-TNT expectation in the 26.3 {id, properties} form).
Suites after the fix: save:: 82 passed, entity_nbt:: 128 passed,
block_entity_nbt:: 13 passed, tnt:: 16 passed. Five failures remain,
all verified pre-existing on the clean base tree (04328e5f, via
stash): save:: a_chunk_from_before_1_18_is_brought_to_the_current_layout
(blending_data), save:: older_records_of_a_current_world_upgrade_as_they_are_read
(palette id), entity_nbt:: frame_missing_facing_loads_down_and_valid_directions_round_trip
(item_frame_from_nbt returns None), and the two Silent-potion sound
tests in tnt.rs. None is touched by this change; all are pre-existing
test debt in other subsystems.

version_history (batch 3): vanilla's fresh-world level.dat has NO
version_history tag, but Horizon always wrote [4903]. The level.dat
writer now preserves the loaded value: it appends the current version
only to a non-empty history (the upgrade case) and writes the tag only
when the history is non-empty. A fresh world stays tag-less, as vanilla
writes it.

## CI run 43704536595 completed (2026-11-15)

Run 43704536595 on main@f56493a6 (the fully-closed tree, pre-v6.7)
completed in ~1h59m, conclusion success. Profile **6/0/11**:
block_properties newly passes, plus first_person, food, lan, light,
resource_pack. D1/D2/D3 pass (417/417); D4 blocked (the staged-candidate
stamp quirk); D5 blocked (no harness-subset.txt on that tree — it
predates f45667f7); D6 blocked (no playtest). save_roundtrip and
command_exec still fail at launch, as expected on the pre-v6.7 tree.
Artifacts at ~/workspace/mcir-ci/26.2-run-43704536595/.

Next run: dispatched on main AFTER the save-roundtrip fixes merge, so
one run verifies v6.7 (the launch fix) and the palette/version_history
fixes together.

## save_roundtrip PASSES locally (2026-11-15)

Full local run after the three batches: `save round-trip parity ok
(0 unexplained differences)` — 2962 -> 0. The game launched on
llvmpipe under Xvfb, saved, and quit cleanly; vanilla loaded Horizon's
save with no new log problems (no "Not a map", no "Failed to create
block entity"); all 79 block-entity VANILLA VIEW comparisons match.
The harness removed its run directory on success (exit 0). Run log:
save-roundtrip-diagnosis/save-roundtrip-local4b.log (reused the v0
world and fast binaries from the first attempt, which had failed at
launch only because it ran without a display — winit needs DISPLAY
even offscreen; re-ran under xvfb-run).

The 4 pre-known vanilla scene-command errors (decorated_pot sherds,
BlockState "No key Name" on the TNT/anvil summons) persist unchanged
— out of scope here. The vanilla_login bot timeout also persists
(player .dat still unexercised under harness load — recorded harness
debt, not this work).

## CI run 43710204861 completed — v6.7 PROVEN (2026-11-15)

Run 43710204861 on main@140d9570 (v6.7 + tag check report-only)
completed in ~1h45m, conclusion success. Profile **6/0/11** (same six:
block_properties, first_person, food, lan, light, resource_pack).
D1/D2/D3 pass; D4/D5 blocked (the staged-candidate stamp quirk);
D6 blocked (no playtest). Artifacts at
~/workspace/mcir-ci/26.2-run-43710204861/.

v6.7 PROVEN on CI: save_roundtrip ran to a real comparison —
`save round-trip parity FAILED (2962 unexplained differences)`, the
exact local count (2403 "Not a map" occurrences) — with no xkbcommon
or Vulkan signature anywhere in the harness logs. The launch crash is
fixed on the runner. This tree predates the save fixes (merged to main
as d2971498 after the run started); the next run measures them.

command_exec: the cushion fix is verified — the abort moved past the
/summon minecraft:cushion failure to the next fixture blocker:
`Java fixture control failed: /execute if predicate effect_view:always:
[]` (corpus tools/mob_effect_predicate_corpus.py). Fixture-side, for
the harness/behaviour work, not the save work.

## CI run 43719972622 dispatched on main (2026-11-15)

Run 43719972622 dispatched on main@e3e2b8d0 — the first run to measure
the merged save fixes (v6.7 launch fix + palette/entity-state map-form
fix + heightmap/DespawnDelay harness rules + version_history
preservation + tag check report-only). Expect save_roundtrip to pass
(it passes locally with 0 unexplained differences) and command_exec to
reach real comparisons; D5 should now judge a real verdict (it fails
honestly until datafix and the rest of the E7 eight are green).

## CI run 43719972622 completed — save_roundtrip PASSES on CI (2026-11-15)

Run 43719972622 on main@e3e2b8d0 (the merged save fixes) completed in
~1h44m, conclusion success. Profile **7/0/10** — save_roundtrip now
PASSES on CI (was FAILED with 2962 unexplained differences on the
pre-fix tree, run 43710204861). Passing: block_properties, first_person,
food, lan, light, resource_pack, save_roundtrip. D1/D2/D3 pass; D4/D5
blocked (the staged-candidate stamp quirk); D6 blocked (no playtest).
Artifacts at ~/workspace/mcir-ci/26.2-run-43719972622/.

D4 implication: save_roundtrip is now green in a published profile; D4
still needs datafix green (T4 oracle work, paused). D5 will fail
honestly once a profile is stamped for the judged tree — the E7 eight
are not all green (datafix, chunk_generation, command_parse,
component_patch, entity_components, item_model still fail).

command_exec still fails on the effect_view predicate fixture blocker
(`Java fixture control failed: /execute if predicate
effect_view:always: []`), unchanged from run 43710204861. Fixture-side,
for the harness/behaviour work.

## Tag-literal triage: the 7 dead tags ported + D1 gating (2026-11-15)

Owner decision 2026-11-15 20:43 NPT: gate the tag-literal check and
triage the 7 dead tags the report-only run found (all inherited from
the 26.3 base, all absent from 26.2's tag data). Each tag was checked
against the decompiled 26.2 oracle; none exists in 26.2 under another
name. Branch `gate/tag-gating`.

| Tag | 26.2 form (oracle) | Fix |
|---|---|---|
| `extendable_maps` | no tag check; cartography gates on saved data, lock, scale | lookup removed; paper branch is `!locked && scale < 4` |
| `brewing_potion_inputs` | hardcoded 4-item list in `PotionSlot.mayPlaceItem` | tag disjunct removed; `is_bottle` covers exactly the four |
| `conduit_effect_block` | hardcoded `VALID_BLOCKS` (prismarine, prismarine bricks, sea lantern, dark prismarine) | shared `is_conduit_frame_block` predicate at all 3 call sites |
| `entities_can_teleport_to` | `isSafe`: not liquid, not `#fire` | negated `#fire` check (`block_in_tag`); rule extracted as `spread_spot_safe` |
| `consumable_does_not_teleport_to` | no tag check in `randomTeleport` | disjunct removed (dead code under 26.2 data) |
| `shulker_does_not_teleport_to` | no tag check in `teleportSomewhere` | check removed; shulkers may cling to bedrock in 26.2 |
| `causes_suffocation` | shape-based `isSuffocating` (`blocksMotion` + full collision shape) | the owed suffocation port (below) |

Four fixes change behaviour (cartography extension, conduit
activation, spreadplayers placement, suffocation) and carry
fail-old/pass-new regression tests. Three are dead-code removals under
26.2 data (brewing, fox, shulker): their tests pin the 26.2 contract
(brewing's proves the tag is not consulted even when the test data
names it; the shulker test asserts bedrock clinging) and the audit
proves the literals are gone.

**Suffocation port.** `BlockFacts.blocks_motion_names` computes the
motion set from `legacy_solid` per name, minus cobweb and bamboo
sapling (the oracle's exclusions). One block mixes legacy solidity
across its states — `resin_brick_wall` (`up=false` + all-sides-`none`
states are not solid) — but walls are never full-collision blocks, so
the outcome is unaffected; the default state is solid, matching the
oracle's per-name `defaultBlockState().blocksMotion()`. The oracle now
classifies the default predicate as `motion_full`, exports `motion.json`,
and runs a single binding with no tag reload; both fixtures read
`motion.json`; both drivers add it to their fixture paths and the
suffocation corpus runs once instead of three tag variants. The port
also fixes the `VanillaSuffocationOracle` compile failure, which had
oracle-failed every harness listing it as a helper (suffocation,
registry-collision-scan, arrow, free-space, native-scene-collision,
native-scene-motion, world-collision-check, world-edge-backoff).

Observed, NOT fixed (out of scope for the tag work): the fox's ground
predicate in `random_teleport` uses `full_collision` where the oracle
uses `blocksMotion` — they differ for fences, walls and similar
blocks. Recorded here, not changed.

**Gating.** D1 now FAILS on dead production tags and BLOCKS when
prepared tag data is absent. Era rule: releases before 1.13 (data
version < 1519, the Flattening) have no tag system, so the audit skips
them — derived from the tree's own `DATA_VERSION`, no release list
hardcoded. Verified locally: audit clean on 26.2 (0 dead production,
5 dead test-only fixtures, unchanged and deliberate); D1 passes on
26.2 with the flip; a synthetic 1.0 pin skips; 1.21.11 stays blocked
on its missing regen artifacts (the audit runs after the regen checks,
so the flip cannot turn it failed).

## Suffocation harnesses pass locally (2026-11-15)

After the suffocation port, both harnesses that consume the oracle and
the fixtures pass against both 26.2 jars:

- `test_suffocation_parity.py`: 169,174 fixtures, **515,376 lines
  matched on client and server**. The driver re-pins the registry size
  to 32366 states and reads the 26.2 map-form `Name` (both were 26.3
  leftovers: 35723 and `id`).
- `test_registry_collision_scan_parity.py`: **2,804,926 lines matched
  on client and server**.

The oracle's motion set (941 blocks) agrees exactly with the game's
`blocks_motion_names` set computed from `legacy_solid` (941, same
members). These two harnesses are outside the 17-harness CI subset,
so the CI profile is unchanged by them; their value is the suffocation
port's oracle proof and the unblocking of the eight harnesses that
compile the oracle.

## CI run 43738833564 dispatched on main (2026-11-15)

Run 43738833564 dispatched on main@ad9757b6 (the tag-gating merge):
proves the gated D1 (dead production tags fail, absent tag data
blocks, pre-1.13 skipped) plus the seven tag ports on CI. Locally the
audit is clean (0 dead production, 5 dead test-only) and D1 passes
with the flip; suffocation parity passes on both jars.

## CI run 43738833564 completed — gated D1 PROVEN (2026-11-15)

Run 43738833564 on main@ad9757b6 (the tag-gating merge) completed in
~1h37m, conclusion success. Profile **7/0/10** — unchanged from run
43719972622 (block_properties, first_person, food, lan, light,
resource_pack, save_roundtrip): no regressions from the seven tag
ports. **D1 passes with the gating audit**: "tag audit: 710 tags
across 20 registries; scanned 245 literals in 2866 files; 0 dead in
production code, 5 dead in test fixtures". D2/D3 pass; D4/D5 blocked
(the staged-candidate stamp quirk — profile stamped for ad9757b6,
judge HEAD is the staged candidate); D6 blocked (no playtest).
Artifacts at ~/workspace/mcir-ci/26.2-run-43738833564/.

## CI run 43749313660 dispatched on main (2026-11-16)

Run 43749313660 dispatched on main@b3e0ad5e (the pins-move merge):
proves the moved pin gates identically under port-verify v6.8 — the
profile, ci-out logs and verdict publish and judge from pins/26.2/,
and D1/D3 read the diff report from the pin. Locally the recorded-mode
verdict already reproduces run 43738833564 exactly (D1/D2/D3 pass,
D4/D5 blocked on the profile-stamp quirk, D6 blocked); this run is the
CI proof of the same.

## CI run 43749313660 completed — moved pin PROVEN (2026-11-16)

Run 43749313660 on main@b3e0ad5e (the pins-move merge) completed in
~1h48m, conclusion success. Profile **7/0/10** — identical to run
43738833564 (block_properties, first_person, food, lan, light,
resource_pack, save_roundtrip): the move changed no behaviour.
**The moved pin gates identically**: the judge resolved the pin to
`pins/26.2/` (tree: the staged candidate `.horizon-ports/26.2`),
D1 found `regen.json` and D3 found `report.json` in the pin,
D2 read `pins/26.2/ci-out/` — **D1/D2/D3 pass**, D4/D5 blocked (the
same staged-candidate stamp quirk — profile stamped for b3e0ad5e,
judge HEAD is the staged candidate c26f655b), D6 blocked (no
playtest). Tag audit clean (0 dead production, 5 dead test-only).
Artifacts at ~/workspace/mcir-ci/26.2-run-43749313660/; the pin's
recorded evidence (profile.json, delta-verdict.json, ci-out logs)
now comes from this run.

## Datafix: BlendingDataFix key 4997 -> 4882, fix_1493 empty region files (2026-11-16)

CI run 43749313660 had datafix failing with ~44,346 differences over 6
shards, in exactly two categories. Both are fixed here, verified locally
against the 26.2 jars (6 worlds, 0 differences each) and proven on CI
below.

1. `chunk.blending_data` (~44,330 diffs). Vanilla 26.2 registers
   `BlendingDataFix` at fix version 4882 (`DataFixers.BLENDING_VERSION =
   4882`, read from the 26.2 server jar); 26.3 raised it to 4997. The
   port carried the 26.3 key, which is past 26.2's target DataVersion
   4903, so the fix never ran when upgrading to 26.2: pre-1.18 chunks
   kept the 1.18 `{old_noise}` form (vanilla writes `{min_section: 0,
   max_section: 16}`) and 1.18+ chunks got no `blending_data` at all
   (vanilla writes `{min_section: -4, max_section: 20}`). The fix body
   already matched vanilla 26.2's `updateChunkTag` exactly (skip
   statuses, 384/-64 and 256/0 forms, overworld-only); only the key was
   wrong. Re-keyed to 4882 in `fixes_c.rs` (6c2d4378).
2. Whole region files missing from Horizon's output (16 files over the
   shards, all pre-1.13 worlds: 3 on 1.10.2, 5 on 1.8.9, ...). Vanilla's
   `LegacyStructureFileFix` reads every chunk its structures reach
   (each start plus the `STRUCTURE_RANGE` 8 square) through
   `SimpleRegionStorage`, and `RegionFileStorage.getRegionFile` opens
   the chunk's region file with CREATE: a reached region holding no
   chunk is left behind as a 0-byte file, which the dimension move then
   carries to `dimensions/.../region/`. Verified with a probe that runs
   only the FileFixerUpper step (the three 0-byte files appear) and by
   the structures themselves (the 1.10.2 Mineshaft at chunk (30, 2)
   reaches regions (0,-1), (1,-1), (1,0) through its square). The port
   only rewrote existing region files; `fix_1493` now creates the empty
   files for every touched region (aba08e87).

Local verification (world_probe rebuilt, oracle = 26.2 server jar):
1.8.9 (DV 0), 1.10.2 (DV 512), 1.13.2-flat (DV 1631), 1.15.2 (DV 2230),
1.21.9 (DV 4554), 26.1 (DV 4786) — all ok, 0 differences. The 1.21.9
world was generated fresh by the 26.2 generator; the others were copied
from the e6-1.21.8 cache (same generator, same scene) to skip the
jre-legacy runtime download.
