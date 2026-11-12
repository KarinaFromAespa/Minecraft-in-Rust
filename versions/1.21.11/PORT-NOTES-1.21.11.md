# Port notes: Minecraft 1.21.11

1.21.11 is the template version of the **Components era (E6, 1.20.5–1.21.11)**
and the first version in the programme whose jar is **obfuscated**: Mojang
stopped shipping unobfuscated builds before this era, and 1.21.11 is
specified through its official mapping tables instead. The era's gate was
that the pipeline run on such a release at all — **prepare, end-to-end,
through the mappings front-end**. That gate is met, and this tree is the
result: the pin is set, all six generated tables are regenerated
from 1.21.11's own jar, the parity oracles compile and run against it, the
horizon-prepare exporter bundle compiles and runs against it, the distance
to the neighbouring port is sized and ranked, and the remaining work is
itemised below by cause. The behaviour code in this tree is still the 26.3
base's; the port is **in progress**, not parity-verified.

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
- Completion pass (this update): the oracle bootstrap rework, the
  remaining table regenerations, and the horizon-prepare bundle, with the
  harness subset measured as far as this VM allowed (see below).

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

| Generator | Result |
|---|---|
| `version_consts` | regenerated (`crates/horizon-core/src/version.rs`, +9 −9) |
| `packet_ids` | regenerated (`crates/horizon-protocol/src/ids.rs`, +179 −197) |
| `entity_tracking` | regenerated (`crates/horizon-client/src/lan/tracking.rs`, +17 −23) |
| `lan_command_order` | regenerated (`crates/horizon-client/src/lan/command_order.rs`, +49 −311) |
| `command_suggestions` | regenerated (`crates/horizon-client/src/lan/command_suggestions.rs`, +11 −26) |
| `vanilla_command_order` | regenerated (`crates/horizon-command/src/vanilla_order.rs`, +5 −26) |

`entity_tracking` needed the port's first era adaptation: 1.21.11 predates
the `EntityTypes` registry split, so the decompile step takes `EntityType`
(the class that held the registry) and the generator reads either source —
the same fix the 26.1 port made, lifted into this tree's
`tools/port_regen.py` and `tools/generate_entity_tracking.py`.

## The oracle bootstrap rework (done)

The era-opening port's blocker is closed. The parity oracles bootstrap
1.21.11's registries the way 1.21.11 does — determined from its named jar
(vanilla's own `WorldLoader` is the reference), not by renaming the 26.3
shape into it:

- `RegistryDataLoader.load` in 1.21.11 is the **synchronous three-argument
  form** `load(ResourceManager, List<RegistryLookup<?>>, List<RegistryData<?>>)`,
  returning `RegistryAccess.Frozen` — no Executor, no `.join()`. Both load
  calls (worldgen and dimension registries) were adapted in `ParityData`
  and `VanillaCommandParseOracle`.
- **There is no component-initializer mechanism in 1.21.11**: no
  `BuiltInRegistries.DATA_COMPONENT_INITIALIZERS`, no
  `DataComponentInitializers` class. Item default components are fixed at
  item registration during `Bootstrap.bootStrap()`; vanilla's load
  sequence has no post-load apply step. The oracle now only applies the
  static tags (`Registry.PendingTags::apply`), as
  `ReloadableServerResources` does.
- **There are no world clocks in 1.21.11**: no `Registries.WORLD_CLOCK`,
  no `ServerClockManager`, no clock/random-sequence/scheduled-event fields
  on `MinecraftServer`. Timelines do exist (the `TIMELINE` datapack
  registry; the four vanilla ones — day, early_game, moon,
  villager_schedule) but carry environment-attribute tracks only, with no
  `registerTimeMarkers`. The registries export keeps the schema and emits
  empty `clocks` / `clock_time_markers`.
- **Template loading**: no `templatesystem.loader` package, no
  `ResourceManagerTemplateSource`. `StructureTemplateManager` is
  constructed directly from a `LevelStorageSource` access, and
  `listTemplates()` enumerates the ids.
- `CommandSourceStack`'s 1.21.11 constructor takes nine arguments
  (`CommandSource, Vec3, Vec2, ServerLevel, PermissionSet, name,
  displayName, MinecraftServer, Entity`), and `suggestRegistryElements` is
  declared on it with `ResourceKey<? extends Registry<?>>` (no type
  parameter, no predicate filter); the delegate helper is the three-argument
  form. Recipes are not a registry lookup here: the learnable recipes come
  from the recipe manager's loaded recipes.
- Verified against the real jars, in the candidate tree: the command oracle
  in tree mode produces the full command tree (75 root commands); in
  registries mode it exports 1,505 items, 157 entity types, 1,458 learnable
  recipes of 1,470 loaded (12 special), 1,202 templates and the four
  vanilla timelines; in run mode, at permission level 4 all corpus
  commands parse with correct node paths and live suggestions.

### A signed-jar landmine, found and fixed

Running any oracle against the remapped jars died with
`SecurityException: SHA-384 digest error` from the JVM's JarVerifier
(via `JfrProfiler.<clinit>`): remapping rewrites class bytes, but the
named jars kept Mojang's signature block, so the signed manifest digests
no longer matched. `tools/lab.py` gained `strip_signature` (drops the
`.SF`/`.RSA`/`.DSA` entries, keeps only the manifest's main attributes),
called from `remap_stage` for both jars **before** their digests are
recorded into `remapped.json`; the store's live jars were stripped in
place and `remapped.json` updated (client sha1
`1b09cd188272163a4ca3506b205c064e98807958`, server sha1
`99f0600a0d9f05fed1e9955562974550dbdf0dde`). Authentication stays
consistent: `test_rust_parity.verify_artifacts` accepts remapped.json
digests, and the same acceptance was extended to horizon-prepare (below).
The store's raw front-end outputs (`*-named.jar`) were left unstripped —
they are indexing-only and never executed.

### vanilla_command_order: regenerated by the rewire pass

The oracle side of this generator was closed by the completion pass —
its Java half runs and produces the command tree inside the generator —
but the generator then builds the Rust client probe, and
`horizon-protocol` failed with 26 compile errors over the nine
post-1.21.11 packets. The rewire below removed that blocker, and the
generator now runs clean: `vanilla_command_order` regenerated
(`crates/horizon-command/src/vanilla_order.rs`, +5 −26) — the 26.x-only
commands (`compute`, `swing`, `posteffect`) are gone, and the
dedicated-server commands (`ban`, `op`, `whitelist`, `stop`, …) are
absent, as in 26.1's regenerated table: the oracle exports the
integrated-server tree, where they are not registered.

## The protocol packet-set rewire (done)

The nine packets are rewired, each to the mechanism 1.21.11 actually
has, determined from its own named jar (datagen `packets.json` for the
packet set, `javap` and decompiled classes for the semantics), following
the 26.1 completion pass's pattern where it transfers — and departing
from it where the jar says 1.21.11 differs. 1.21.11's play set is
clientbound 139 / serverbound 66; what 26.3 has that it lacks is exactly
the five clientbound and four serverbound packets below.

### Swing: `SWING_ANIMATION` / `PUNCH` → `animate` / `swing`

- `ClientboundSwingAnimationPacket` does not exist in 1.21.11. The
  swing a player sees is `ClientboundAnimatePacket` (id 2 in all three
  versions) with action 0 (main hand) or 3 (off hand): the constants
  `SWING_MAIN_HAND=0`, `WAKE_UP=2`, `SWING_OFF_HAND=3`,
  `CRITICAL_HIT=4`, `MAGIC_CRITICAL_HIT=5` are identical in 1.21.11,
  26.1 and 26.2, verified by `javap` and by decompiled
  `ClientPacketListener.handleAnimate` (0 → `swing(MAIN_HAND)`, 3 →
  `swing(OFF_HAND)`, 2 → `stopSleepInBed`, 4/5 → crit particles).
  Every `SwingAnimation` send and match in the client — remote
  handling, the LAN hub's broadcast, session recording, the
  swing-flush — is now an `Animate` with action 0 or 3. The local
  `first_person::SwingAnimation` component type (a swing's kind and
  duration) is unrelated to the packet and untouched; the classic
  swing model (`LivingEntity.swinging`, `swingTime`) is still a
  behaviour-port item.
- `ServerboundPunchPacket` does not exist in 1.21.11. The client
  sends `ServerboundSwingPacket(InteractionHand)` (serverbound id 60),
  originating from `LocalPlayer.swing` — the constant-pool scan finds
  no other client sender. Every `Punch` send — the attack swing, the
  mining swing, the LAN guest swing, the load-test bots — is now
  `Swing { hand: 0 }` (or the off hand where the arm is known).
- Two latent 26.3 animate-numbering uses were fixed with the rewire
  (26.1's tree still carries them): the hub's wake-up broadcast and
  the containers' critical-hit broadcast used actions 0 and 1/2; they
  are now 2 (wake) and 4/5 (crit / enchanted crit), per the jar's
  constants.

### Attack and interact: `ATTACK` → `Interact` action 1

- `ServerboundAttackPacket` does not exist in 1.21.11. An attack is
  `ServerboundInteractPacket` with action 1 (`ATTACK`):
  `MultiPlayerGameMode.attack` sends
  `createAttackPacket(entity, shiftKeyDown)`, and the server's
  `handleInteract` dispatches it to `player.attack(target)` after the
  range and loaded checks. The Rust `Interact` packet is reworked to
  the 1.21.11 action form — `InteractAction::{Interact, Attack,
  InteractAt}`, with the action VarInt, the interact-at point as
  three f32s and the hand after it (verified against the jar's
  bytecode: `InteractionAtLocationAction.write` writes `d2f` × 3,
  then the hand) — and every attack send (client, bots) and match
  (hub passthrough, anticheat, LAN guest handling) is an
  `Interact` with action `Attack`. The hub's anticheat check for an
  attack now runs inside the `Interact` arm, only when the action is
  `Attack`. `ClientCommand` action 2 (request game-rule values) is
  gone with the game-rule packets: the enum is respawn (0) and
  request stats (1) only, `javap`-verified.

### Spectator: `SPECTATOR_ACTION` → `teleport_to_entity` (clicks stay attacks)

- `ServerboundSpectatePacket` and `SPECTATOR_ACTION` do not exist in
  1.21.11, and neither does 26.1's `spectate_entity`: 1.21.11 has no
  serverbound spectate packet at all. The spectator camera is chosen
  from the spectator menu, which sends
  `ServerboundTeleportToEntityPacket(UUID)` (serverbound id 61) —
  the only sender in the client jar is `PlayerMenuItem`. Clicks do
  **not** switch the camera: decompiled `Minecraft.startAttack` has
  no spectator branch — a spectator's click goes through the attack
  path, with only the local attack and the final swing skipped — and
  `MultiPlayerGameMode.attack` sends the interact-attack for
  spectators too. So the client's spectator click no longer sends
  anything special (it sends the interact-attack, gated like
  vanilla's), the hub's `guest_spectator_action` handler and its
  `camera_pickable` picker are deleted, and the camera is chosen by
  `/spectate` and the teleport-to-entity packet, which the tree
  already had.

### Game rules: `GAME_RULE_VALUES` / `SET_GAME_RULE` → no wire sync at all

- 1.21.11 has no game-rule packet in any state (verified in datagen
  for play and configuration). The client never receives game rules:
  `ClientboundLoginPacket` has no game-rules field, and decompiled
  `Level`, `ClientLevel` and `ClientPacketListener` contain zero
  `GameRules` references — rules live on the server only
  (`ServerPlayer` reads `ServerLevel.getGameRules()`). Both packets
  are removed from the wire: the remote handler no longer applies
  rule values, the LAN guest no longer sends rule changes, and the
  GameRules screen no longer asks a server for its rules. What
  replaces them is what vanilla 1.21.11 has: the screen shows the
  window's own rules (loaded from level.dat on world open, so
  single-player values are real), a gamemaster's Done applies them
  as `/gamerule` commands, and the button is inactive on an external
  server, where there is no rule source to show. The local `/gamerule`
  command implementation and the window's `GameRules` state are
  untouched.

### Confirmed absent: `POST_EFFECTS`, `ADD_TRANSIENT_BLOCK`, `LOW_DISK_SPACE_WARNING`

- None of the three exists in 1.21.11's packet set (datagen), and none
  has an older equivalent. All three are removed from wire paths
  only, as 26.1 did for post-effects: the guest post-effects sends
  (login, respawn, the dirty check in `flush_progress`) are deleted;
  the falling-block transient-block send is deleted (the block update
  alone is what 1.21.11's `FallingBlockEntity.tick` sends); the
  low-disk-space warning send is replaced by a console log on the
  dedicated server. The local machinery stays inert: the post-effects
  chains still render and save, the transient-block queue still ticks
  and renders (it simply never receives entries), and the low-disk
  detection still feeds the local toast. The two functions left
  without callers (`add_transient_block`,
  `native_world_game_rule_change`) carry `#[allow(dead_code)]` with a
  note, so the tree stays warning-free.

### The adapter

The cross-version adapter is reworked for the 774 pin on the same
pattern: the `swing_id` punch branch, the swing-animation down
conversion, the interact split (which mapped the pin's interact onto
the newer `attack` packet) and the 26.1 spectate step are deleted;
`step_1_21_11`'s `serverbound` conversion is deleted (the pin IS
1.21.11, so its interact crosses as-is); `world::animate` is identity
for actions 0/2/3/4/5. The per-version packet-id tables are still
keyed to the pinned 777 (26.3) — re-basing them is the adapter-ladder
job, not this one — and the adapter tests are updated to the
1.21.11↔26.2 renumbering (swing 60↔63, keep_alive 27↔28/43↔44,
resource_pack 48↔49), with packets 26.2 lacks dropped.

### Oracles and harnesses touched

- The world-options harness is difficulty/lock only: the fixture, the
  `VanillaWorldOptionsOracle` (its `SetGameRule`/`GameRuleValues`
  imports referenced classes that do not exist in 1.21.11) and the
  harness itself (18 cases → 6) all drop the `set`/`values` cases.
- `KnockbackClient` (the knockback harness's Java client) sends the
  1.21.11 forms: the attack is an Interact packet decoded from its
  bytes (entity id, action 1, not sneaking — the packet's public
  constructors take an `Entity`, as with the player-command packet
  already in that file) and the swing is
  `ServerboundSwingPacket(MAIN_HAND)`.

### Verification

- `cargo check --workspace --all-targets`: **0 errors, 0 warnings**
  (`CARGO_BUILD_JOBS=2`; the full-workspace check at default jobs OOMs
  on this VM's 6 GB).
- `vanilla_command_order` regenerated cleanly once the protocol
  compiled (see above) — **6 of 6 tables**.
- `cargo test -p horizon-protocol`: **418 passed, 23 failed**, every
  failure accounted for by family:
  - **19 cross-version adapter tests** — the ladder's per-version
    packet-id tables, layout conversions and registry tables are still
    keyed to the pinned 777 (26.3); under the 774 pin their expected
    ids are off by the drift between the two (e.g. a 26.2 swing maps
    to nothing because the 777-keyed table has no swing counterpart;
    keep_alive maps to 42, the 26.3 id, instead of 43). Seven of the
    nineteen are tests this pass rewrote to assert the correct
    774↔776 mappings; they fail because the tables are stale, not
    because the expectations are wrong. Re-basing the ladder is the
    adapter-regen job's, not this pass's.
  - **2 legacy item tests** (`item::v1_20_3`, the decorated-pot
    component encoding) — the same un-regenerated legacy datafix
    family.
  - **2 sandbox-network environment failures** (the DNS resolver and
    the dead-address connection test, both blocked by the sandbox's
    proxy/permissions) — the same pair every version reports here.
- Three `packets::play::tests` that hardcoded 26.3 packet ids
  (`change_game_mode`, `tracked_waypoint`, `update_attributes`) were
  updated to the regenerated 774 ids (4, 136, 129 — verified against
  datagen `packets.json`) and pass.
- Harness: `command_parse` — the one harness whose measured cause was
  the probe build — now runs to a real verdict (fail, 7,147
  mismatches; see the harness table). The other eight unmeasured
  harnesses need the full game binary and were not reached on this VM;
  the CI `port-verify` workflow produces the full profile.

## The adapter ladder re-base (done)

The cross-version adapter ladder is re-based to the 774 pin, following
the 26.1 line's ff7d657 method. The ladder is consistent only whole:
every version's tables were regenerated, not just one.

- **Prepared store reused**: the 11 legacy versions (1.20.2–1.21.11)
  prepared for the 26.1 re-base — verified download + mappings
  front-end remap + datagen — were reused as-is; no re-prepare was
  needed. The 26.3 packets report arbitrated the packet renames.
- **Packet-id tables regenerated** (12 files):
  `tools/generate_packet_id_adapter.py` against the 774 pin, with
  arbitrated renames (e.g. `horse_screen_open` → `mount_screen_open`
  for 1.21.3–1.21.10; `game_profile` → `login_finished` and
  `set_carried_item` → `set_held_slot` for 1.20.6/1.21.1; `bundle` →
  `bundle_delimiter` and `client_intention` → `intention` for
  1.20.2/1.20.3; `resource_pack` → `resource_pack_push` for 1.20.2).
  `v1_21_11.rs` is identity (play 139/66); 26.2/26.1.2 packets
  genuinely absent in 1.21.11 (`game_rule_values`,
  `low_disk_space_warning`, `attack`, `set_game_rule`,
  `spectator_action`/`spectate_entity`) map to None.
  `debug_sample_subscription` is deliberately left unrenamed (the 26.1
  precedent).
- **Registry-id tables regenerated** (14 files):
  `tools/regenerate_registry_id_adapters.py` against the 774 pin. One
  driver rename dropped: `command_argument_type` `minecraft:color` →
  `minecraft:team_color` — 1.21.11 still calls it `minecraft:color`
  (the rename happened in 26.x). The 774 component registry (104
  entries, last id 103 `shulker/color`) is a strict subset of 775's;
  the six 26.1-only components (`additional_trade_cost`, `dye`, the
  four mob sound variants) are dropped under the pin.
- **Production code re-keyed** (`item.rs`, `item/skip.rs`,
  `item/hash.rs`, `item/hash/gameplay.rs`, `item/hash/raw.rs`,
  `entity_data.rs`, `item/v1_20_3.rs`, `item/v1_21_4.rs`
  TOOLTIP_FLAGS/ADDITIONAL, `item/v1_21_5.rs` snip holder) and **all
  test fixtures re-keyed** (the `item/v1_*` fixtures regenerated
  through the converter itself; the adapter translate /
  client_session / world / layout tests re-keyed by hand from the
  1.21.11 and 26.2 reports). The rewire's files were ported onto, never
  copied over: the `play_sound` removals and swing renumberings the
  26.1 completion made do not apply here (the 1.21.11 rewire did its
  own equivalents).
- **The seven rewire-rewritten tests** asserting the correct
  774↔776 mappings now pass with the tables right; none was rewritten
  back.
- **The 2 legacy item failures** (v1_20_3 pot encoding) are fixed: the
  item-conversion code this job re-keyed covers them.
- **The slot_display trap**: the SlotDisplay codec still uses the 26.3
  numbering (item=4, smithing_trim=8, composite=10), but 1.21.11's
  slot_display registry has 8 entries in a different order (item=2,
  smithing_trim=5, composite=7). The generator's old→pin mapping is
  identity for the 8-entry versions and 11→8 for the 26.x versions —
  both wrong for the codec as it stands (the 1.21.4 smithing test read
  its composite/item bytes as WithAnyPotion). Five generated tables
  are hand-corrected to map old → the codec's 26.3 numbering instead
  (`v1_21_8_ids` — inherited by 1.21.3/4/5/6 — `v1_21_10_ids`,
  `v1_21_11_ids` to the 8→11 Runs table; `v26_1_2_ids`, `v26_2_ids`
  to Same), each with a comment. When the behaviour port rewires the
  codec to the 1.21.11 layout, these five tables must be regenerated
  to old→pin.
- **Pin-older-than-client**: the 26.2-client tests follow the 26.1
  tree's ed18214 pattern — 26.2 has every block/item/entity 1.21.11
  has (verified: the down tables cover all 29,671 pinned states,
  1,505 items, 157 entities), so no stand-in applies for that client
  and the stand-in test is deleted; the chunk test uses calcite
  (24485 → 27160) and the chunk-section test is renamed to
  `..._is_renumbered` at 15 bits (oak_slab 13330 → 13128).

### Verification

- `cargo test -p horizon-protocol`: **438 passed, 2 failed** (was
  418/23). The only failures are the sandbox-network pair (out of
  scope, the same pair every version reports here). The 19 adapter
  failures and the 2 legacy-item failures are all closed. (440 tests
  total vs 441 before: the deleted stand-in test.)
- `cargo check --workspace --all-targets`: **0 errors, 0 warnings**
  (`CARGO_BUILD_JOBS=2`). The lib-only check first surfaced one
  warning — `four_texts` in `item/hash/raw.rs` orphaned when the
  26.3 SignText arm (kinds 118/119) was dropped, since sign text is
  not a component in 1.21.11; the helper is deleted (the 26.1 tree
  carries the same orphan).

## horizon-prepare and the exporter bundle (done, with three era fixes)

The full exporter bundle compiles against 1.21.11 (16 server mains plus
4 client mains, with every transitive oracle) and horizon-prepare runs
against the named jars end-to-end — **completed in the candidate, 553.9 s,
exit 0**, producing `.local/horizon-client/1.21.11` (1,505 items with
their default components, the worldgen bundle, the six block-behaviour
files, the datafix tables). The adaptations, by family:

- **A shared `VanillaStateCodec`** (`mods/horizon/.../parity/`):
  `BlockState.FULL_CODEC` does not exist in 1.21.11 — it has only the
  palette codec — so the flat `{id, properties}` form the fixtures
  consume is assembled from the registry and the state's property map
  (encode/decode), following the 26.1 pass's helper. Ten oracle files
  were converted to it.
- **Landing / movement / placement / simple-block oracles**, following
  the 26.1 pass's adaptations where they transfer: `FarmBlock` is
  1.21.11's farmland class (no `baseBlock` field — turn-to-dirt hardcodes
  dirt); the bounce path is `updateEntityMovementAfterFallOn`, with
  `SlimeBlock` and `BedBlock` as the bounce declarers; bounce
  restitution / fall-distance-reduction fields and the
  `SUPPRESSES_BOUNCE` tag do not exist yet; `ServerLevel`'s constructor
  takes an Executor and a `RandomSequences`; `Level` has abstract
  `fuelValues()` and `potionBrewing()` and a `getShade` on its
  `BlockAndTintGetter`; `NoiseSettings` is a four-argument record and
  `NoiseRouter` has fifteen density functions (the all-zero router is
  built directly — `NoiseRouterData.none()` is protected here);
  `RepeatingPlacement.count` is protected (invoked reflectively);
  `PlacementModifier` positions come from `getPositions`, not `modify`.
  `VanillaSimpleBlockOracle` was restructured on 26.1's version:
  features place through `FeaturePlaceContext`, the decode returns
  `ConfiguredFeature`s, `RegistryInfoLookup` returns `RegistryInfo`
  records, and block-state providers are parsed inline (not a registry
  here). `VanillaMobDataExporter` uses `EntityType`, not the later
  `EntityTypes`; `VanillaGameplayFactsExporter` reads light dampening as
  `getLightBlock()`; `VanillaCreativeTabsExporter` reads item tags as
  `getTags()`.
- **The data-fix tables exporter under JDK 21**: the exporter hooks
  `BlockStateData.register` by bytecode patching, which upstream does
  with the Class-File API — `java.lang.classfile` does not exist in JDK
  21 (it arrived in JDK 22 as a preview), and 1.21.11 pins Java 21. The
  hook is reimplemented in the exporter as a small class-file editor
  (constant-pool append for the `record()` methodref, six hook bytes
  prepended to `register`'s code, every pc-bearing table shifted).
  Verified end-to-end: the exporter records 1,695 block-state lines and
  its own replay check (replaying the calls must reproduce vanilla's
  finished tables) passes.
- **Client model oracles on the classic API**: the 26.x `cuboid` /
  `sprite` / `dispatch` packages do not exist in 1.21.11. The binding
  oracle uses `BlockModelDefinition` (record, with `CODEC`,
  `instantiate`, `simpleModels`), `BlockStateModel`'s unbaked roots and
  the multipart `Unbaked` with `visualEqualityGroup`; the cuboid oracle
  uses `BlockModel.fromStream`, `SimpleUnbakedGeometry.elements()`,
  `BlockElement`/`BlockElementFace`/`BlockElementRotation` (with the same
  single-axis / Euler rotation value shapes), and `TextureSlots.Data`
  slots (a `Reference` target or a `Material` whose sprite is
  `texture()`; 1.21.11 materials have no translucency flag, and elements
  carry a boolean shade flag, not a shade-direction override — both
  emitted as what vanilla has). `BlockStateDefinitions.
  definitionLocationToBlockStateMapper()` is not public here; it is
  called reflectively.
- **A note on names**: 1.21.11 already uses `net.minecraft.resources.
  Identifier` (Mojang renamed `ResourceLocation` in this release); the
  oracles use it throughout.

### horizon-prepare's own era adaptations (in this tree)

- `crates/horizon-prepare/src/artifacts.rs` — `verify()` accepts the
  remapping stage's authentication: a jar whose digest matches
  `remapped.json`'s record, and whose recorded mapping-table sha1 is the
  manifest's own mapping download, is accepted for the client jar and the
  inner server jar. Without this, `verify()` can never pass on an
  obfuscated release, whose prepared jars are the remapped ones by
  design. The manifest, bundler and library digests are still checked
  against Mojang's pins.
- `crates/horizon-prepare/src/survival.rs` — 1.21.11's stem classes carry
  no `supportBlocks` TagKey field (their `mayPlaceOn` hardcodes
  farmland), so the binding binds no rule for the four stem blocks rather
  than failing the whole prepare or inventing one. The rule form for
  them — and for the nether fungus/roots blocks, whose 1.21.11 placement
  classes (`FungusBlock`, `RootsBlock`) use hardcoded
  nylium/mycelium/soul-soil checks — belongs to the behaviour port.
- `crates/horizon-prepare/src/lib.rs` — the datagen install accepts
  1.21.11's single `reports/items.json` (each entry carries the same
  `components` object) when the later per-item components report
  directory does not exist; and a worldgen registry that is empty in
  both jars (`material_rule`, `material_condition`,
  `block_state_provider` — registries this version does not have) is
  written as an empty bundle instead of failing the reference check,
  because the generator's required reads expect the file to exist.

## Harness subset (measured on this VM)

Seventeen harnesses were dispatched; the VM is slow and shared, and each
harness takes on the order of fifteen minutes here. Nine were measured;
the rest are **unmeasured** — never inherited silently — and the CI
`port-verify` workflow on main can produce the full profile afterwards.
Every measured harness fails, each for a named cause below; none is a
pass.

| Harness | Verdict | Cause (measured) |
|---|---|---|
| `block_properties` | fail | The Rust probe build (`cargo build --release -p horizon-world --example world_probe`) timed out at 900 s on the contended VM; the oracle side ran. No code verdict. |
| `chunk_generation` | fail | The oracle does not compile against 1.21.11 (33 errors). `ChunkPos.pack` is `asLong` here, and the 26.x generation-stage API it drives does not exist: no `ChunkStatus.TERRAIN`, no `generator.buildTerrain` — 1.21.11 generates through the separate `createBiomes` / noise / surface / carver / feature stages. Porting that oracle is part of the chunk-generation behaviour work. |
| `command_exec` | fail | The oracle does not compile (37 errors). The `ChunkPos` accessor renames and the two spam-threshold server overrides are fixed in this tree; what remains is the integrated-server bootstrap — `LevelSettings` / `DifficultySettings`, `PrimaryLevelData`'s constructor, and the world-preset lookup differ. |
| `command_parse` | fail | **Re-measured by the rewire pass**: the probe builds now (the 26 protocol errors are gone) and the harness runs to a real verdict — 7,147 command parse mismatches. Permission 0: 34,460/34,461 commands match; permission 2: 30,895/34,461. Sample causes: the Rust command tree still holds 26.x commands (`compute`, `posteffect`) that 1.21.11 rejects; registry lookups fail (`minecraft:oak` in `worldgen/feature` — the Rust side's registry data is the 26.3 base's); error-message/cursor differences (`time set  1`). Behaviour-port work, now measurable. |
| `component_patch` | fail | The oracle runs but dies at runtime: `ClassNotFoundException: net.minecraft.core.component.Removed` — a 26.x component type that does not exist in 1.21.11. |
| `datafix` | fail (all 6 shards) | `datafix_worlds.py` exits 1 for the whole legacy chain (1.8.9 → 26.2): at measurement time the cross-version adapter tables for 1.20.2–1.21.11 were not regenerated yet (the 26.1 residual). The re-base has since landed (see above), so this harness needs re-measuring; its verdict may change. |
| `entity_components` | fail | The harness's own fixture builder breaks on 1.21.11 data: `test_entity_components_parity.py` expects `criterion['conditions']['entity']['predicate']` to be a mapping, but the advancement condition shape differs here (`TypeError`). |
| `first_person` | fail | The oracle does not compile: it is written against the 26.x first-person swing system — `FirstPersonHandsAndItems`, `SwingState`, `swing(InteractionHand, SwingAnimation, boolean)` — which does not exist in 1.21.11; the classic swing model is a behaviour-port item (same family as the `SWING_ANIMATION` packet). |
| `food` | fail | The oracle does not compile: `VanillaFoodOracle`'s fake `ServerLevel` constructor (the 1.21.11 eleven-argument form, as fixed in the bundle oracles). |
| `hotbar`, `item_model`, `lan`, `light`, `options`, `resource_pack`, `save_roundtrip`, `sound_event` | **unmeasured** | Not reached on this VM. |

## What remains, by cause

### Behaviour work from the diff (not started)

The 815 ranked items are the port's work list. Largest first: the play
packet renumbering and the two/three removed packets per direction, the
registry id-space shifts (item, block, sound), and the 4,154 changed
classes. The behaviour code in this tree is still the 26.3 base's.

### Harness-oracle era ports

- `VanillaChunkGenerationOracle`: the 1.21.11 generation-stage pipeline
  (no `TERRAIN` status, no `buildTerrain`).
- `VanillaCommandExecOracle`: the integrated-server bootstrap
  (`LevelSettings`, `PrimaryLevelData`, world-preset lookup).
- `VanillaFirstPersonOracle`: the classic swing model (no
  `FirstPersonHandsAndItems` / `SwingState`).
- `VanillaFoodOracle`: the fake `ServerLevel` constructor.
- `VanillaComponentPatchOracle`: the `Removed` component type does not
  exist in 1.21.11.
- `test_entity_components_parity.py`: the advancement-condition shape
  for entity predicates.
- A survival-rule form for single-block / composed below-checks (stems
  on farmland; fungus/roots on nylium, mycelium, soul soil), so the four
  stem blocks and the nether vegetation get bound rules instead of none.
- `datafix`: re-measure now that the adapter re-base has landed (its
  recorded cause was the un-regenerated 1.20.2–1.21.11 tables).

### Full harness profile

The remaining eight harnesses are unmeasured locally; the CI
`port-verify` workflow on main completes the profile on GitHub runners.

## Status

Protocol packet-set rewire done (nine 26.3-only packets rewired to
their 1.21.11 mechanisms from the jar); 6 of 6 tables regenerated;
adapter ladder re-based to the 774 pin (all 19 adapter failures and
both legacy-item failures closed); workspace compiles with 0 errors
and 0 warnings; protocol tests 438 pass / 2 fail (sandbox-network
pair only). Oracle bootstrap rework done and verified against the
jars; horizon-prepare bundle compiles and runs against the named jars
end to end; 9 of 17 harnesses measured (all fail, by the causes
above), 8 unmeasured. The behaviour port (815 ranked items) is the
remaining work.
Status in [VERSIONS.md](../../VERSIONS.md): **in progress**.

## CI profile — port-verify v5, run 43114684827 (2026-11-11)

The first full 17-harness profile on GitHub runners, against the
rewire-only tree (candidate commit 306a153c5245, main@d8479b1 — BEFORE
the adapter re-base 2e2ef22, so a fresh run from the re-based tree is
still owed). `prepare: ok`; the front-end remap works on CI too
(lab-prepare imported 10,291 client classes from the remapped jar);
horizon-prepare completed in 29.6 s; protocol compiles clean (0 errors).

**Profile: 2 pass / 2 oracle-no-longer-compiles / 13 fail.**
Passes: block_properties, lan. (The "15 newly failing" count is a
no-baseline classification artifact.) Notably, block_properties PASSES
on clean hardware — its local failure was a probe-build timeout under
VM contention, not a tree defect. The two oracle failures are the
known era ports: chunk_generation (33 javac errors, no
`ChunkStatus.TERRAIN`/`buildTerrain`) and command_exec (29 errors,
integrated-server bootstrap + the interact-packet constructor shape).
Food fails at oracle compile too (VanillaFoodOracle javac exit 1).
The remaining 12 fails are the behaviour-port work (815 ranked items):
the command tree and registry data are still the 26.3 base's.
Artifacts kept at `~/workspace/mcir-ci/1.21.11v5-prerebase/`.

## CI profile — port-verify v5, run 43127256427 (2026-11-12, post-re-base)

The fresh run from the re-based tree (main@6587c35, adapter re-base
2e2ef22 included). **Profile: 2 pass / 2 oracle / 13 fail — identical
to the pre-re-base profile.** Same passes (block_properties, lan),
same oracle failures (chunk_generation, command_exec). The re-base
closed the protocol-test failures (438/2) but moved no harness
verdict, which is consistent: the harness failures are behaviour-port
and oracle-port work, not adapter-table work.

**datafix cause corrected.** The completion notes blamed stale
adapter tables; that was wrong. Both the pre-re-base and post-re-base
runs fail datafix identically, before any comparison runs: the
fixture generation starts a 26.1 server jar, which needs Java 25
(class file 69.0), but the CI runner installs only the version's own
Java (21, class file 65.0) — `UnsupportedClassVersionError`, 600 s
timeout, all six shards fail the same way. `datafix_worlds.py`
resolves 1.20.5+ servers to plain `java` on PATH. So datafix is
currently **unmeasured in CI for pre-26.x versions**, not failing on
behaviour. Queued fix: install multiple JDKs in port-verify (the
version's own for oracles, Java 25 on PATH for the fixture servers).
Artifacts kept at `~/workspace/mcir-ci/1.21.11-postrebase-v5/`.
