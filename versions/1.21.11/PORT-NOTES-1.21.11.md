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

## CI profile — port-verify v6.4, run 43149789732 (2026-11-12, multi-JDK validation)

The live test of the v6.4 multi-JDK fix (main@d1658a0, ~1h47m).
**Profile: 2 pass / 2 oracle / 13 fail — the same shape as both v5
runs.** Same passes (block_properties, lan), same oracle failures
(chunk_generation 33 errors, command_exec 29 errors — unchanged
counts, the T4 work). D2 passes in CI for this tree: cargo check
--workspace --all-targets 0 errors / 0 warnings, protocol tests 440
with none failing (the 2 local "failures" are the sandbox-network
pair, which pass with real network).

**The multi-JDK fix worked — and exposed the next layer.** datafix
no longer fails with UnsupportedClassVersionError: the fixture
generation now runs its 26.x servers under Java 25 and passes. The
harness then reaches the oracle compile, which fails: the
VanillaDataFixOracle is still the 26.3 oracle and does not compile
against the 1.21.11 client jar (javac exit 1, all six shards). Note
the ordering: main() generates worlds BEFORE compiling the oracle,
so the v5 runs never reached the oracle compile at all — this
failure is not a regression, it is the next blocker, and it is
oracle-port work of the same class as chunk_generation and
command_exec. (Its javac errors go to a build-dir log the artifacts
do not carry; the T4 agent reproduces them locally.) datafix's
profile status is "fail", but its true failure mode is
oracle-compile — the classifier sees a generic failure because the
errors are hidden in the build log. Food and item_model also fail
at oracle compile in this run.

**Publish-profile shard bug found in this run's verdict.** The
judge reported D4 blocked ("not measured: datafix") even though all
six datafix shards ran and failed: publish_profile.py reads
parity.py's state.json, which records shards as `datafix#1..6`
with no aggregate entry, and the gate judges the name `datafix`.
Fixed in tools/publish_profile.py (collapse_shards: the aggregate
is pass iff all shards pass, fail if any fails, else the first
non-pass status; the shard entries stay alongside it). Tested on
synthetic states (all-fail, all-pass, mixed-oracle) and end to end
through the gate: D4 now judges datafix (fail) and passes when
both datafix and save_roundtrip pass. The next CI run will carry
the fix.

**Gate backfill landed with this run's processing.**
versions/1.21.11/ now carries tier.txt ("T"), the committed
diff-26.1-1.21.11/report.json (from the 1.21.11 agent's store:
815 items, ranks 10/492/247/64/2) and an honest work-items.json
(0 closed, 0 accepted) — D3 now fails with the correct totals
instead of blocking. Still missing: regen.json (the store holds
regen files only for the 26.3→1.21.11 and self diffs, not
26.1→1.21.11) and regen-determinism.json — both need a fresh
1.21.11 regen, queued. Artifacts kept at
`~/workspace/mcir-ci/1.21.11-v6.4-multijdk/`.

## T4 oracle ports — behaviour stream, 2026-11-19 (branch behaviour/1.21.11)

Re-surveyed from scratch with versions/1.21.11/.local/t4-survey.py
(driver-faithful javac invocations, JDK 25 javac --release 21): 148
errors across 10 oracles; 6 already compiled (component_patch, hotbar,
options, block_properties, lan, command_parse). All 10 ported, full
re-survey 16/16 at 0 errors. Prepared data recovered from the old
workdir's lab-prepare output (.local/minecraft/1.21.11/); no fresh
prepare run was needed. CI proof run 44446975616 dispatched on the
branch at fbb1c14 (in flight).

- Food (1 error): the fake ServerLevel constructor takes 11 args in
  1.21.11; appended `new RandomSequences()`. 58 oracle files share the
  10-arg pattern — only the 17-harness compile closure was fixed.
- EntityComponents (4): EntityPredicate lives at
  advancements.criterion (not advancements.predicates.entity); the
  11-arg ServerLevel super; deleted the getNextEntityId override
  (entity ids come from Entity.ENTITY_COUNTER here).
- PackResources (1): Pack.open() returns a single PackResources.
- SoundEvent (2): resolve() returns WeighedSoundEvents;
  Sound.getAttenuationDistance takes no args (base distance only).
- Suffocation (2 + 7-file change set): 1.21.11 has no
  causes_suffocation tag; the default predicate is
  blocksMotion() && isCollisionShapeFullBlock, verified from the jar's
  bytecode. Oracle exports motion.json; the suffocation definition and
  both probe fixtures bind the single motion set; the game-side
  suffocation code (tag lookup in mobs/suffocation::bind, cushions.rs)
  is behaviour-port work, not T4. Registry count asserted 29671.
- ChunkGeneration (33): 11-arg OracleCache/OracleLevel supers; deleted
  getNextEntityId/getOverworldClockTime/uncachedBiomeResolver
  overrides (the biome path works through getChunkSource); registry
  bootstrap replaced with the already-ported synchronous 3-arg
  RegistryDataLoader.load + layers.replaceFrom(WORLDGEN) pattern;
  RandomState.create via the (settings, noise lookup, seed) overload;
  ChunkPos.asLong static; TERRAIN replaced by NOISE/SURFACE/CARVERS
  (fillFromNoise/buildSurface/applyCarvers, each with its own region);
  createForNormal takes 4 args.
- DataFix (21): no FileFixer system in 1.21.11 (pre-1.16 structure
  files stay in place; LegacyStructureDataHandler reads them lazily).
  level.dat is fixed in place (stamp mirrors vanilla's save, including
  Series from DataVersion.series()); old world layout (region/entities/
  poi/saved data per dimension, playerdata/advancements/stats/);
  generators() reads level.dat's WorldGenSettings; ChunkMap
  getChunkDataFixContextTag takes the generator's registry key;
  saved-data switch trimmed to the 9 types 1.21.11 defines.
  tools/datafix_worlds.py VERSIONS capped at 1.21.11. The Rust-side
  datafix behaviour (skip the 26.x file fixes, trim the saved-data
  table) is behaviour-port work.
- CommandExec (29): 8-arg MinecraftServer ctor, 7-arg LevelSettings,
  datapackWorldgen(), 4-arg PrimaryLevelData, WorldData cookie for
  WorldLoader.load, public tickServer, EntityType, createInteractionPacket,
  ClickType, level-owned random sequences/weather/data storage,
  teleport(TeleportTransition), static ChunkPos.asLong, 2-arg map ticking.
- FirstPerson (21): rewritten for the classic swing model — no
  SwingState, no FirstPersonHandsAndItems. The swing lives on
  LivingEntity (swinging/swingingArm/swingTime/attackAnim/oAttackAnim),
  advanced by the real updateSwingTime and sized by
  getCurrentSwingDuration; the hand heights live on the real
  ItemInHandRenderer behind a shell GameRenderer (startUseItem calls
  gameRenderer.itemInHandRenderer.itemUsed on a consuming use). The
  Rust first-person fixture is still 26.x-shaped; the classic-swing
  fixture port is behaviour work.
- ItemModel (34): the older model system — BakedQuad/BlockModel/
  ItemTransform/ItemModelGenerator under renderer.block.model, no
  geometry/cuboid/sprite split, non-generic SpecialModelRenderer.Unbaked,
  8-arg submit with the display context, Unbaked records without
  transformation, MaterialSet + SpriteGetter instead of MaterialBaker,
  default components set through Item's private components field,
  bundle selection through the (items, weight, selected) constructor,
  LayerRenderState (int[] tints, TextureAtlasSprite particle icon,
  ItemTransform applied to a fresh pose, List<BakedQuad>).
- ComponentPatch (runtime): 1.21.11 has no Removed class;
  DataComponentPatch.map holds Optionals, empty for removals. The
  oracle reflected on Removed.INSTANCE and died before running;
  patchText unwraps the Optional. PatchedDataComponentMap.set returns
  previous.orElse(prototype), so the Rust set over an unsanitized
  removal marker returns Ok(base) instead of MissingPrototype.
- EntityComponents driver: the trigger's entity is a list of loot
  conditions in 1.21.11 (bare object in 26.3); the driver accepts both
  shapes and uses the 1.21.11 predicate keys (components, type).

T4 commits on behaviour/1.21.11: aa4ecb1 (batch 1: food,
entity_components, pack_resources, sound_event, suffocation Java +
drivers, chunk_generation), 3e8a93d (datafix), 8ead9be (command_exec),
29ea9a9 (suffocation Rust, after cargo check), 4a1de47 (first_person),
f236b5c (item_model), 19682d1 (component_patch), fbb1c14
(entity_components driver).

Rank 1 (10 items, CLOSED 2026-11-19)
- Items 2, 3, 4, 5, 7, 8 verified current: protocol (removed packets
  have no Rust types; step_1_21_11.rs exists; ids.rs current),
  version_consts (--check passes), packet_ids, lan_command_order (236
  nodes, real server jar) and command_suggestions (122 nodes) all
  normalized-identical; entity_tracking 157/157 after the generator
  regex fix (045f7f6: single-line registers spanned into the next
  entry).
- Item 9 save-format (2b1a914): 1.21.11 has no FileFixerUpper
  (net.minecraft.util.filefix arrived with DataVersion 4772 in 26.1)
  and no FILE_FIXING_REQUIRED backup status (BackupStatus is only
  NONE/DOWNGRADE/UPGRADE_TO_SNAPSHOT, verified from the client jar).
  Deleted fix_1493/4772/4773/4899, level_dat_to_saved_data and their
  helpers (move_file, resolve, delete, dimensions, read_level,
  write_level, read/write_saved_data, regex_move); requires_file_fixing
  always false; upgrade_band is data fixes only (the oracle's band is
  too); upgrade_world_data_with follows the pre-26.x layout (overworld
  at the root, DIM-1/DIM1, playerdata/, advancements/, stats/);
  generators and current_world_gen_settings read level.dat's
  WorldGenSettings; saved_data_type keeps only the 9 types 1.21.11's
  DataFixTypes defines (matches the ported oracle's savedDataType).
  needs_file_fixes is always false now. NOT in item 9: the save.rs
  settings store still writes world_gen_settings.dat / world_clocks.dat
  / game_rules.dat as saved-data files (26.x) instead of level.dat's
  Data — that rewire is rank-2 save_roundtrip work.
- Item 10 facts (f46c381): Realms test headers 1.21.11/774/Java 21;
  production code already reads version.rs.
- Item 6 vanilla_order (verified 2026-11-19, no change): the
  INTEGRATED vanilla tree (.local/command-parse-parity/tree.json, 75
  root children, from the command-parse oracle) against the Rust tree
  (client_probe fast build, finished 24m27s after two SIGTERM/quota
  retries) through generate_command_order.py gives 27 nodes, byte-
  identical after rustfmt to the committed vanilla_order.rs.
- regen.json + regen-determinism.json (D1, 2026-11-19): all six
  generators run fresh reproduce the committed tables byte-identical
  (version_consts, entity_tracking, lan_command_order,
  command_suggestions via port_regen.run_generators; packet_ids with
  --report .local/datagen-1.21.11/reports/packets.json — the stock
  default .local/datagen/out/reports/packets.json does not exist in
  this tree; vanilla_command_order via the oracle tree + fast
  client_probe fixture). regen.json sits beside report.json
  (diff-26.1-1.21.11/regen.json); regen-determinism.json in
  pins/1.21.11/ (committed == fresh, no overrides). D1 now passes
  the regen/determinism steps locally; the remaining D1 steps
  (prepared data at .local/horizon-client/1.21.11, tag audit) need
  the CI judge's prepare layout — the tag audit run locally against
  the client jar's tag data finds the known 31 dead production tags
  (see rank-2 tag triage below).
- Rank 1 closed 10/10 in work-items.json on 2026-11-19.
- Item 1 components (9e92320): DATA_COMPONENT_TYPES 122 -> 116; dye
  colours from the item name (facts, loom, sheep dyeing); only wolves
  keep sound variants. Two oracles the T4 survey missed needed ports
  in the same files: VanillaSheepDyeOracle used DataComponents.DYE
  (does not exist in 1.21.11) and a 9-arg LocalPlayer ctor (7 in
  1.21.11) — both fixed, and the harness expectation is 384 lines
  (patch loop runs once). VanillaFinalizeSpawnOracle still does NOT
  compile (13+ errors: ServerLevel 11-arg ctor, getNextEntityId,
  cubemob package, DataComponentInitializers, EntityTypes) — a T4
  gap, NOT in the 17-harness CI profile, to port as follow-up work;
  its DATA_SOUND_VARIANT_ID reads for pig/cow/chicken/cat were
  already removed (the fields do not exist; the reads threw at
  runtime). Also noted, not item 1: SYNCHRONIZED keeps
  sulfur_cube_archetype (a 26.2 registry, no report item covers it)
  and lacks minecraft:timeline (in 1.21.11's
  RegistryDataLoader.SYNCHRONIZED_REGISTRIES) — rank-2 registry work.

CI run 44448815383 (on a7a9ede, T4 state, completed 2026-11-19):
profile 5 pass / 1 oracle / 11 fail (v6.4 was 2/2/13). Passing:
block_properties, component_patch, food, lan, resource_pack.
Oracle-fail: command_exec. D2 passes (0 errors, 0 warnings; 440
protocol tests). D1 blocked on the owed regen.json; D3 fails on 0/10
rank-1 closed (port in progress); D4/D5/D6 blocked (stamp quirk, no
harness-subset.txt, no playtest.md — all record-time items).
component_patch green on CI settles the local re-run: no local
re-run needed.

CI run 44460096417 (on 8fa812d, items 1/9/10 state, completed
2026-11-19, 1h54m): profile 5 pass / 1 oracle / 11 fail — unchanged
from 44448815383 (same passing set: block_properties,
component_patch, food, lan, resource_pack; oracle-fail:
command_exec). Items 1/9/10 move no harness: item 1's component
removals sit inside the already-green component_patch, item 9's
save changes sit inside save_roundtrip/datafix (failing for other
reasons), item 10 is test-only. D2 passes (0 errors, 0 warnings;
440 protocol tests). D1 blocked on the owed regen.json; D3 fails on
0/10 rank-1 closed (port in progress — rank-1 code is 9/10 done,
item 6 awaiting the client_probe build); D4/D5/D6 blocked (stamp
quirk, no harness-subset.txt, no playtest.md — deferred per owner).
Recorded in pins/1.21.11/ (profile.json stamped 8fa812d,
delta-verdict.json).

CI run 44484665862 (on a3a429f, A-batches 1-3 state, completed
2026-11-19, ~2h): profile 5 pass / 5 oracle / 7 fail in the
17-harness framing (23 entries with the datafix shards). Passing:
block_properties, component_patch, food, lan, resource_pack —
unchanged from the two earlier runs. Real comparisons (4):
command_parse (6337 mismatches), datafix (15056 differences over
7 worlds), hotbar (889/905), options (options.txt ordering/content
diffs). Oracle-side (5): chunk_generation (the vanilla oracle
crashes generating a chunk — "Requested chunk unavailable during
world generation" in Beardifier, a fixture/setup issue, not a
comparison), command_exec (SeedAgent.java uses
`java.lang.classfile`, which does not exist at `--release 21` —
the seed agent needs a Java 21-compatible form; 26.2 never hit
this because its Java is 26.x), entity_components, first_person,
item_model (the oracle java run exits non-zero; its own log is
not in the artifact, so the cause is unrecorded). Launch/setup
(3): light and save_roundtrip fail on "missing motion tag" (the
Rust probe/game fails to launch), sound_event on the missing
sounds.json (the known unwired-sounds gap). D1 now FAILS (not
blocked): regen.json exists, so the tag audit ran — 26 dead
production tags remain (tag triage is part of the port). D2
passes (0 errors, 0 warnings; 440 protocol tests). D3/D5 blocked
on file lookup, not on substance: v6.8 resolves the pin when one
exists, and the pin held no work-items.json or harness-subset.txt
— both are now recorded in the pin (work-items.json at the
record-time state, rank2.closed = 90; harness-subset.txt is the
E6 eight). D4 blocked (stamp quirk: profile stamped a3a429f,
judge HEAD the staged candidate). D6 blocked (no playtest.md —
deferred per owner). Recorded in pins/1.21.11/ (profile.json
stamped a3a429f, delta-verdict.json, work-items.json,
harness-subset.txt).

CI run 44499421617 (on 336b435, A-batches 6-7 state, completed
2026-11-19, ~2h20m): profile 5 pass / 1 oracle / 11 fail in the
17-harness framing (5/18 over 23 entries with the datafix shards).
Passing: block_properties, component_patch, food, lan, resource_pack
— unchanged from run 44484665862. Real comparisons (4):
command_parse (6337 mismatches, identical), datafix (15054
differences over 7 worlds, vs 15056 — noise), hotbar (889/905,
identical), options (options.txt ordering/content diffs, 185 vs
194 lines). Oracle-side (5): chunk_generation (same Beardifier
"Requested chunk unavailable" oracle crash), command_exec
(SeedAgent.java `java.lang.classfile` at `--release 21` — the
known harness-tooling follow-up), entity_components, first_person,
item_model (oracle java runs exit non-zero). Launch/setup (3):
light ("missing motion tag"), save_roundtrip (server closed the
connection), sound_event (missing sounds.json). D1 FAILS on the
same 26 dead production tags. D2 passes (0 errors, 0 warnings;
440 protocol tests). D3/D5 blocked on file lookup — this run was
dispatched on 336b435, before the pin held work-items.json or
harness-subset.txt (both added in the 44484665862 record). D4
blocked (stamp quirk: profile stamped 336b435, judge HEAD the
staged candidate). D6 blocked (no playtest.md — deferred per
owner). Recorded in pins/1.21.11/ (profile.json stamped 336b435,
delta-verdict.json, work-items.json at record-time state,
rank2.closed = 120).

CI run 44518329820 (on 9b7791e, A-batches 14-19 state, completed
2026-11-19, ~1h22m): profile 5 pass / 1 oracle / 11 fail in the
17-harness framing (5/18 over 23 entries with the datafix shards).
Passing: block_properties, component_patch, food, lan, resource_pack
— unchanged from run 44499421617. Real comparisons (4):
command_parse (6337 mismatches, identical), datafix (15054
differences over 7 worlds, shard 1 — identical), hotbar (889/905,
identical), options (options.txt ordering/content diffs, 185 vs
194 lines — identical). Oracle-side (5): chunk_generation (same
Beardifier "Requested chunk unavailable" oracle crash), command_exec
(SeedAgent.java `java.lang.classfile` at `--release 21` — the
known harness-tooling follow-up), entity_components, first_person,
item_model (oracle java runs exit non-zero). Launch/setup (3):
light ("missing motion tag"), save_roundtrip (server closed the
connection), sound_event (missing sounds.json). D1 FAILS on the
same 26 dead production tags. D2 passes (0 errors, 0 warnings;
440 protocol tests). D3 FAILS with real totals now (the pin held
work-items.json at dispatch): rank-2 372 of 492 neither closed nor
accepted at the pin snapshot (120 closed). D4/D5 blocked (stamp
quirk: profile stamped 9b7791e, judge HEAD the staged candidate).
D6 blocked (no playtest.md — deferred per owner). Recorded in
pins/1.21.11/ (profile.json stamped 9b7791e, delta-verdict.json,
work-items.json at record-time state, rank2.closed = 189).

Two earlier runs landed the same day with the same profile 5/1/11
and the same verdict shape: 44506660231 (on 5184978, A-batches 8-9
state, D3 rank-2 402 open at its pin snapshot) and 44513629292 (on
7090c16, A-batches 10-12 state, D3 rank-2 372 open). Both
superseded by 44518329820; their profiles are not recorded in the
pin.

Rank 2 (492 items, in progress 2026-11-19)
- Method: each item's named methods intersected with the diff store
  (rank2-triage.json, workdir scratch): 189 items with no changed named
  methods (quick verify) and 303 with real changes. Verification is a
  normalized `javap -c -p` bytecode diff (javap_diff.py, workdir
  scratch) of the item's classes, 26.1 vs 1.21.11, plus a Rust-side
  check against the jars. No-change items are verified, not assumed:
  the Java diff is read for every item. Batches of 10 in id order
  ("A" batches = no-change items first).
- A-batch 1 (70, 73, 91, 108, 135, 144, 150, 154, 172, 192), verified
  2026-11-19, ported the same day:
  - 70 no-change (network): Connection access/lambda noise;
    FriendlyByteBuf ChunkPos helpers renamed (same single-long wire
    format); 1.21.11's extra readVec3/readLpVec3 already exist in the
    Rust (Vec3d, LpVec3).
  - 73 no-change (core): Direction lambda order; 26.1 holder-attached
    data components are a server-side API 1.21.11 predates; the
    Rust's registry-sync model (NBT per entry) IS 1.21.11's wire form.
  - 91 no-change (network/codec): ByteBufCodecs$1..$34 byte-identical
    (+0/-0); flagged on constant-pool churn only.
  - 108 CHANGED (item models): 26.1 has a per-model `transformation`
    on every Unbaked record + `bake(BakingContext, Matrix4fc)`;
    1.21.11 has neither. Removed from decode + bake in item_model.rs,
    item_bake.rs, item_model/foreign.rs (+ the foreign wrap test): the
    `transformation` field, `optional_transformation`,
    `Transformation.EXTENDED_CODEC`, `compose`, and the `parent`
    chain in `Resolve::node` (layers are identity now; the display
    transform still applies at render time via `layer_matrix`). A pack
    carrying a `transformation` field parses with it ignored.
    Block-model display transforms untouched (separate feature, exists
    in 1.21.11 too).
  - 135 CHANGED (args.rs, one line): the slot range at offset 300,
    size 8, is `"villager."` in 1.21.11, `"mob.inventory."` in 26.1.
    Every other range verified identical. (Side find: 1.21.11's
    `getCustomTabSugggestions`, three g's.)
  - 144 no-change (nbt): SNBT grammar identical; 1.21.11 adds a
    convenience `acceptAll(Stream)` overload.
  - 150 CHANGED (levelgen): NoiseChunk's `fullNoiseDensity` /
    `getInterpolatedDensity` are 26.1-only (the Rust has no
    equivalent); NoiseRouterData's `noNewCaves` is 1.21.11-only but
    the Rust builds routers data-driven from the 1.21.11 datapack
    JSON (verified: nether.json is the noNewCaves form). WorldOptions
    codec field renamed `generate_structures` -> `generate_features`
    in save.rs (read + write) + 2 comment updates. The
    `rename(generate_features, generate_structures)` fix at key
    4771 is inert at 1.21.11 (fixer only runs fixes <= DATA_VERSION
    4671); fixes above 4671 are inert dead code, left alone.
  - 154 CHANGED (test instances): test-instance padding is 26.x-only
    (`GameTestInstance.padding()` + `getPadding()` /
    `getTestBoundingBox()` / `getTestBounds()`). Removed the padding
    plumbing: `TestDef.padding` (field, decode, `test_holder_padding`),
    the padding params on `renderable_box` / `structure_box` /
    `start_corner`, the bounds inflation in `place_test_structure`
    and `test_bounding_box` (now just the structure box), and the
    padding offsets in test_locate.rs + structure_blocks.rs. The
    boxes test now asserts the unpadded values. `export_test` stays a
    stub (both versions' non-dev behaviour). test_instance_edit.rs
    PADDING (UI layout) untouched.
  - 172 CHANGED (components): `BlocksAttacks.bypassedBy` is a TagKey
    in 1.21.11 (bare identifier on the wire) vs a HolderSet in 26.1.
    The wire skip in skip.rs now reads it as `optional(ident)`; the
    damage-reduction `type` stays a holder set in both versions.
    Bundle weight semantics fold into item 85 (26.1 fallible
    `DataResult<Fraction>` vs 1.21.11 plain `Fraction`; observable
    difference only on overflow). ItemContainerContents slot shape
    identical (comment fixed). WrittenBookContent's generation cap is
    enforced by the recipe's `allowed_generations` default (0,1).
  - 192 CHANGED (particle light): 1.21.11 has no `LightCoordsUtil`;
    glowing particles return constant 240 (sky 0, block 15) where
    26.1 does `withBlock(light, 15)`, and non-glowing
    FlyTowardsPosition adds t^4*240 to the SKY channel where 26.1's
    `addSmoothBlockEmission` adds it to the BLOCK channel. Added
    `Light::Bright240` (VaultConnection, SculkCharge/Pop) and
    `Light::AgeEmissionSky4` (Enchant, Nautilus). The shared
    `Block15` / `AgeEmission4` variants stay for their own items —
    check each user against its item's Java diff (the withBlock-15
    -> constant-240 pattern is likely systematic).
- A-batch 2 (206, 213, 214, 219, 236, 237, 241, 244, 245, 252),
  verified 2026-11-19, ported the same day:
  - 206 no-change (renderer): RenderPipelines registry differs
    (Mojang-internal GPU registry; the Rust builds its own wgpu
    pipelines). Chest sheet switch-map + GUI render-state renames.
  - 213 no-change (server/level): ServerBossEvent UUID + `setDirty`
    (26.x dirty-tracking); broadcast packets identical.
  - 214 no-change (server): `ReloadableServerResources` 26.x
    DataComponentInitializers vs 1.21.11's simpler load (consistent
    with the T4 gap); `WorldStem` WorldDataAndGenSettings -> WorldData.
  - 219 no-change (item properties): 26.1-only lazy
    DataComponentInitializers / `craftRemainder(ItemStackTemplate)` /
    `requiredFeatures`; the 5-entry `CRAFT_REMAINDERS` table verified
    identical in both versions (all use the `(Item)` overload).
  - 236 no-change (camel): CamelModel ctor refactor (Mojang-internal);
    `CamelAnimations` UNCHANGED between versions; CamelAi server-side.
  - 237 no-change (chunks): `LevelChunkSection` fluid counting —
    26.1 counts fluids separately (second wire short), 1.21.11 folds
    them into nonEmptyBlockCount (one short). The adapter already
    handles both directions; the client never consumes the counts.
  - 241 no-change (multiplayer): 26.1 sanitizes the server MOTD,
    1.21.11 assigns directly; the Rust parses status JSON raw.
  - 244 no-change (screens): InBedChatScreen gating moved into
    ChatScreen in 26.1 (same behaviour); the Rust has its own in-bed
    chat. GuiGraphicsExtractor vs GuiGraphics rename.
  - 245 no-change (loot): `LootTable.validate` helper vs inlined loops
    (same validation); `VisitedEntry` generics bound only.
  - 252 CHANGED (post-processing): 1.21.11's `hasPostProcess`
    (predicate `Blocks.always`) marks SELF on brown/red mushroom +
    magma_block; soul_sand has none. 26.1's `getPostProcessPos`
    marks mushrooms SELF, soul_sand + magma ABOVE (bubble columns).
    `postprocess_self` gains magma_block, `postprocess_above` is now
    empty. (Out of scope: the `legacy.rs` 12-block structure list —
    vanilla structure placement uses flags with 16 set in both
    versions, so the path never fires there; same in both, not a
    1.21.11 regression.)
- A-batch 3 (263, 264, 275, 279, 285, 311, 313, 314, 315, 316),
  verified 2026-11-19, ported the same day:
  - 263 CHANGED (tags): 26.1-only tags with live Rust references —
    `loom_dyes`/`loom_patterns` (1.21.11's loom slots check
    `instanceof DyeItem` / `has(PROVIDES_BANNER_PATTERNS)`; the tag
    gates dropped, the data-driven lookups ARE the checks),
    `cauldron_can_remove_dye` -> `dyeable` (identical 6-item
    contents; DYED_COLOR check already in cauldron.rs),
    `wolf_collar_dyes` -> DyeItem check (`dye_from_name`, now pub),
    `supports_frogspawn` (1.21.11's `mayPlaceOn` is water-only; tag
    branch dropped). EnchantmentTags TRADES_*_SPECIAL exist as
    constants in 1.21.11 but have no data in either jar; no Rust
    refs. TagLoader is server-side.
  - 264 no-change (equine): spawn/aging/breeding/pathfinding diffs,
    all server-side (26.1 zombie horses return an offspring from
    `getBreedOffspring`, 1.21.11 returns null).
  - 275 no-change (RenderSystem): 26.1 `GpuDevice.presentFrame()` vs
    1.21.11 `GLFW.glfwSwapBuffers` — Mojang GPU-backend plumbing;
    the Rust never touches RenderSystem.
  - 279 no-change (dialogs): DialogAction's 3 constants identical
    (only the CODEC field's declared type differs); ItemBody
    ItemStackTemplate vs ItemStack (same shape; comment fixed).
  - 285 no-change (fluids): `isWaterHole` helper vs inlined checks
    (server-side flow); FluidState StateHolder refactor;
    `typeHolder()` vs `holder()`; `isFull()` 26.1-only (the Rust's
    fluid mesher reads heights from the level property).
  - 311 CHANGED (particles): ONLY PAUSE_MOB_GROWTH and
    RESET_MOB_GROWTH are 26.1-only particle types. 1.21.11's
    AgeableMob emits only HAPPY_VILLAGER; the age-lock particle
    spawn is removed (`make_age_locked_particle` ->
    `tick_age_lock_timer`, timer decrement only), the enum variants
    and the render case removed. The item-particle wire decode
    (`Template::read`, item + count + patch) already matches
    1.21.11's `ItemStack.STREAM_CODEC` (comment fixed). CORRECTION:
    the whole age-lock MECHANIC is 26.x-only — 1.21.11's AgeableMob
    has no `isAgeLocked`/`setAgeLocked`/`ageLockParticleTimer`, and
    1.21.11's Items has no GOLDEN_DANDELION. Its removal
    (`age_locked` field, `set_age_locked_data`,
    `can_use_golden_dandelion`, the golden-dandelion interaction,
    AgeLocked NBT, `cannot_be_age_locked`) belongs to item 23
    (rank 2, CHANGED, covers mobs/ai.rs + AgeableMob), not item 311.
  - 313 no-change (block entities): TypedInstance / ChunkPos
    methods-vs-fields / `typeHolder()`; SculkCatalyst/Sensor
    byte-identical.
  - 314 no-change (entity storage): ctor order + `ChunkPos.pack` ->
    `toLong`. Server-side.
  - 315 no-change (container screens): `SnapbackData` is 26.1-only
    (the Rust has no snapback code); GuiGraphicsExtractor renames.
  - 316 no-change (HumanoidModel): 26.1-only per-slot armor mesh
    system (`ADULT/BABY_ARMOR_PARTS_PER_SLOT`,
    `createBabyArmorMesh(Set)`); 1.21.11-only `setAllVisible`. NO
    ArmPose or setupAnim changes. The Rust builds humanoid models
    from `entity-models.json` (1.21.11 jars).
- Cross-item flags: `ItemStackTemplate` is 26.x-only (absent from
  both 1.21.11 jars); the JSON/wire shape is usually the same as
  ItemStack, so most uses need comment-only fixes — verify per item.
  Fixed so far: item_model.rs (stack_from_template doc),
  block_entity_nbt.rs:219, block_entity_item.rs:271, dialog.rs:101,
  entity_data.rs (Item doc), item.rs (Template codec doc + 2 test
  names), item_sync.rs (2 docs). Still owed by their own items:
  crafting.rs (recipe result type), stations.rs (smithing),
  mod_content_tests.rs. Glowing-particle light: check every
  `Light::Block15` user against its item's Java diff.
- Tag triage (D1-gating): the tag audit run 2026-11-19 against the
  client jar's tag data (552 tags) finds 31 dead production tags +
  5 dead test-only fixtures (deliberate, never fail). Item 263's
  ports fix 5 of the 31 (loom_dyes, loom_patterns,
  cauldron_can_remove_dye, supports_frogspawn, wolf_collar_dyes).
  26 remain to triage (teleport tags, motion/suffocation tags,
  block-behaviour tags, sulfur_cube_*); each needs its 1.21.11
  equivalent verified against the jars.
- Rank-2 A-batches 1-3: 30 items closed (9 changed, 21 no-change).
  work-items.json rank2.closed = 30 on 2026-11-19.
- A-batch 4 (317, 319, 320, 321, 322, 323, 324, 325, 326, 327),
  verified 2026-11-19, ported the same day:
  - 317 no-change (block-entity render context): 26.1
    `blockModelResolver` vs 1.21.11 `blockRenderDispatcher` +
    extra `itemRenderer`; `sprites` vs `materials` (echoes item
    108's BakingContext finding). The Rust has no such record.
  - 319 no-change (sounds): BlockableEventLoop ctor flag (26.x),
    `getChannelDebugString` vs `getDebugString` rename,
    `getSoundCacheDebugStats` 26.1-only. The Rust has its own
    audio engine.
  - 320 CHANGED (SoundEvents): 26.1-only (10): GOLDEN_DANDELION_
    USE/UNUSE (item 23) + 8 HORSE_*_BABY events. 1.21.11-only (22):
    the standalone CAT/CHICKEN/COW/PIG events + WOLF_STEP — 26.1
    has NO standalone cat/chicken/cow/pig events (variant methods
    only); 1.21.11 has only `registerWolfSoundVariants`. Systematic
    audit of all 1913 hardcoded sound-id literals against the
    1838-entry 1.21.11 registry: three real misses, all ported.
    (a) Baby-horse sounds: the Rust played
    `entity.baby_horse.{ambient,angry,breathe,death,eat,hurt,land,
    step}`, which exist in NEITHER version (26.1 has
    `entity.horse.*_baby`, 1.21.11 has none — Horse returns the
    adult events unconditionally, babies at 1.5x voice pitch).
    Removed the branches in ai.rs (ambient/hurt/death -> kind
    sounds), equine.rs (make_mad angry, gallop breathe, step, +
    the now-unused `let baby`), mounts.rs (eat), living.rs (land).
    (b) Baby-wolf sounds: `variant_sound` built
    `entity.baby_wolf.{kind}` — no such events (no BABY_WOLF_*,
    no baby set in the 7-variant SoundSet). Baby branch removed;
    baby wolves use their variant set at the baby voice pitch.
    The non-baby variant logic is correct for 1.21.11
    (`entity.wolf_{variant}.{kind}` matches the data files; step
    falls back to `entity.wolf.step`, which exists). (c) Pig eat:
    `entity.pig.eat` is in NEITHER registry; 1.21.11's Pig has no
    playEatingSound at all. Pig removed from `eating_sound`
    (falls to None). Kept: turtle/polar-bear/nautilus baby sounds
    (all exist in 1.21.11), copper golem + happy ghast sounds
    (verified against the registry). Flag: the Rust's parrot
    imitation list has 39 events, the registry 40 —
    `entity.parrot.imitate.ender_dragon` is missing (vanilla's
    `imitateNearbyMobs` includes the ender dragon); belongs to the
    parrot entity item.
  - 321 no-change (chunk storage): RegionFile access flags +
    ChunkPos methods-vs-fields; ChunkBuffer writes the same 4 zero
    bytes + version id in both; SectionStorage renames only.
  - 322 no-change (weather renderer): the 26.x GPU render-backend
    rewrite (BufferBuilder + RenderPass vs MultiBufferSource);
    `renderInstances` itself is IDENTICAL between versions
    (verified method-body diff; only a 26.1 empty-list guard).
    The Rust renders weather with its own wgpu code.
  - 323 no-change (pack resources): 26.1 extracts
    `topPackDir`/`getResource`/`listResources`/`getNamespaces`
    overloads, 1.21.11 inlines the same resolution; 26.1's
    `getNamespaces` additionally checks `isDirectory`.
  - 324 no-change (AI behaviours): `is(EntityType)` vs
    `getType() == type` (same semantics); MoveToTargetSink
    byte-identical. Server-side.
  - 325 no-change (world): Difficulty `getKey()` is 1.21.11-only
    (Java API, not wire-visible); InteractionHand gains an id +
    STREAM_CODEC in 26.1 but ordinals 0/1 are the same wire form;
    LockCode ItemInstance vs ItemStack rename; Stopwatches
    SavedDataType ctor Identifier vs String (same name).
  - 326 no-change (VertexConsumer): 26.1 `putBlockBakedQuad` /
    `putBakedQuad` with QuadInstance + the NEW BakedQuad vs
    1.21.11 `putBulkData` with the OLD BakedQuad — the 26.x
    model-system refactor; the core vertex math is the SAME. The
    Rust builds quads with its own mesher.
  - 327 no-change (StreamCodec): interface order, ctor order,
    lambda naming. Compiler noise; the Rust has its own codecs.
- A-batch 5 (328, 329, 331, 332, 333, 334, 335, 336, 337, 338),
  verified 2026-11-19, all no-change:
  - 328 no-change (allay): Brain provider refactor (26.1 static
    BRAIN_PROVIDER + `makeBrain(Brain$Packed)` vs 1.21.11
    `brainProvider()` + `makeBrain(Dynamic)` + AllayAi.makeBrain);
    SENSOR_TYPES identical; THROW_SOUND_PITCHES identical values.
    Server-side AI.
  - 329 no-change (signs): 26.1 implements the `HangingSignBlock`
    / `PlainSignBlock` interfaces (26.x-only); placement logic
    identical except `FluidState.is(WATER)` vs `getType() ==
    WATER` (same semantics).
  - 331 no-change (zombie nautilus): same Brain refactor;
    ZOMBIE_NAUTILUS_VARIANT wrapped in `EitherHolder` in 1.21.11
    (same holder value). Server-side AI.
  - 332 no-change (phys): AABB$Builder gains a `defined` flag in
    26.1 (the Rust has no AABB builder); `LP_STREAM_CODEC` is a
    Vec3 constant only in 26.1 but EXISTS in 1.21.11 as
    FriendlyByteBuf.readLpVec3/writeLpVec3 (item 70); Vec3
    STREAM_CODEC is 3 doubles in both.
  - 333 no-change (ticks): ChunkPos pack/toLong renames only.
    Server-side.
  - 334 no-change (pack screen): access flags + ctor order only;
    the Rust has its own pack screen.
  - 335 no-change (entity-data serializers): the REGISTRATION
    ORDER differs — 26.1 registers CAT/COW/PIG/CHICKEN_SOUND_
    VARIANT (22/24/29/31), 1.21.11 lacks all four, so every
    serializer after POSE (20) shifts down (HUMANOID_ARM 42->38).
    Verified the adapter ALREADY handles this: the V1_21_11 Runs
    table in adapter.rs matches the jar-derived order exactly
    (up: (0,0,22), (22,23,1), (23,25,4), (27,30,1), (28,32,11)).
    The DYE_COLOR serializer (43) has no 1.21.11 equivalent —
    only produced by the Rust's own LAN server, never read from a
    1.21.11 server. SynchedEntityData/DataItem: access flags only.
  - 336 no-change (packet listener $1): synthetic-class mismatch
    — 26.1's `$1` is a switch-map holder, 1.21.11's is an
    anonymous InteractPacket handler; the comparison is
    meaningless, the logic lives in the outer class (another
    item). Server-side.
  - 337 no-change (player list): ctor order + requireNonNull +
    access flags; UserWhiteList byte-identical. Server-side.
  - 338 no-change (debug sync): `ChunkPos.containing(BlockPos)`
    static vs `new ChunkPos(BlockPos)` ctor (same construction).
    Server-side.
- Rank-2 A-batches 1-5: 50 items closed (10 changed, 40 no-change).
  work-items.json rank2.closed = 50 on 2026-11-19.
- A-batch 6 (339, 340, 341, 342, 343, 344, 345, 347, 348, 351),
  verified 2026-11-19, ported the same day:
  - 339 no-change (fish): access flags; `LivingEntity.is(TagKey)`
    vs `getType().is(TagKey)` (same semantics); Salmon$Variant
    CODEC declared type (same codec); `is(Object)` -> `is(Block)`.
    Server-side.
  - 340 no-change (set_name): MAP_CODEC -> CODEC + `codec()` ->
    `getType()` (the 26.x loot-function registration refactor);
    `ResolutionContext`/`ComponentUtils.resolve` vs
    `CommandSourceStack` + `ComponentUtils.updateForEntity` (same
    change as item 172's WrittenBookContent). The Rust's set_name
    does not model entity resolution (no table uses `entity`), so
    the difference is unobservable.
  - 341 CHANGED (command storage): 1.21.11 writes
    `data/command_storage_<namespace>.dat` (flat under data/,
    `createId` = `"command_storage_" + namespace`, verified by
    `javap -v`); 26.x writes `data/<namespace>/command_storage.dat`.
    The Rust wrote the 26.x layout, so a vanilla-written 1.21.11
    world would never load. `file()` + `load()` rewritten for the
    flat layout (load scans `command_storage_*.dat` files, namespace
    from the filename); the NBT shape is the same. The respawn
    harness glob updated to `command_storage_*.dat` (matches both
    the Rust's new file and vanilla 1.21.11's).
  - 342 no-change (recipe-book screen): `extract*` -> `render*`
    renames (GuiGraphicsExtractor vs GuiGraphics); `ContainerInput`
    vs `ClickType`. Mojang render-API rename.
  - 343 no-change (end crystal): 1.21.11 passes a static RENDER_TYPE
    (entityCutoutNoCull) to submitModel, 26.1 the texture
    Identifier; same texture, same model. CameraRenderState package
    move.
  - 344 CHANGED (phantom + villager textures): the phantom texture
    MOVED in 26.x — 1.21.11 has `textures/entity/phantom.png` and
    `phantom_eyes.png` (flat, verified in the client jar), 26.1 has
    them under `phantom/`. species.rs + render.rs updated.
    VillagerProfessionLayer: 26.1 passes `"baby"` for baby villagers,
    1.21.11 ALWAYS passes `"type"` — there is no `baby/` folder in
    the 1.21.11 jar, so the Rust's baby-villager type texture pointed
    at a missing file. The baby branch is gone (group always
    `"type"`); the baby MODEL (`villager_baby#main`) stays. The
    `/baby/` fallback in the villager-hats lookup stays as a
    pack-defined-texture fallback (never fires for vanilla textures).
    RopesLayer is the same RenderType-vs-Identifier API pattern as
    item 343 (no change).
  - 345 CHANGED (llama textures): 26.1 has `llama_creamy/white/
    brown/gray.png` + `llama_*_baby.png` and a `BabyLlamaModel`;
    1.21.11 has `creamy/white/brown/gray.png` (no `llama_` prefix),
    no baby textures (babies wear the adult coat), and a plain
    `LlamaModel` for babies (verified in the client jar). `texture()`
    now returns `entity/llama/{coat}`; the species defaults updated
    to `entity/llama/creamy`. `decor()` untouched (equipment
    textures; `llama_baby#decor` is a model name from the 1.21.11
    data).
  - 347 no-change (CharacterEvent): 1.21.11's record carries a
    `modifiers` component 26.1's lacks, but a jar-wide check shows
    NO consumer reads it (only KeyboardHandler, which constructs the
    event) — carried, never consumed. The Rust's modifier-less char
    handling is observably identical.
  - 348 no-change (InputConstants): IME/preedit callbacks are 26.x-
    only (1.21.11 uses `glfwSetCharModsCallback`, consistent with
    item 347); raw-mouse support is resolved reflectively in 1.21.11
    (same constant 208901, same behaviour). The Rust uses winit,
    never GLFW.
  - 351 no-change (Display inner classes): access flags only.
- Rank-2 A-batches 1-6: 60 items closed (13 changed, 47 no-change).
  work-items.json rank2.closed = 60 on 2026-11-19.
- A-batch 7 (352, 353, 354, 358, 360, 361, 362, 363, 364, 365),
  verified 2026-11-19, ported the same day:
  - 352 no-change (trial spawner renderer): CameraRenderState
    package move (`state/level/` -> `state/`) only.
  - 353 no-change (villager model + AI): 1.21.11's VillagerModel
    adds `BABY_TRANSFORMER` (0.5 scaling); the Rust renders baby
    villagers via `villager_baby#main` from entity-models.json
    (generated from the 1.21.11 jars), which already encodes the
    baby form. GiveGiftToHero: `getRandom()` -> `random` field.
    Server-side AI.
  - 354 CHANGED (particle light): `Particle.getLightColor` returns
    0 when the particle's chunk is not loaded; 26.x returns
    15728640 (sky 15, block 0). `World::light` keeps the 26.x 0xF0
    fallback for its other callers, so both particle `light_at`
    closures in game.rs now guard with `world.loaded()` and return
    0 for unloaded chunks. TerrainParticle's 1.21.11-only
    `getLightColor` override (fall back to the origin block's light
    when the super value is 0) is a follow-up: the Rust's Particle
    carries no origin pos, so it needs a struct field + plumbing
    through `Particle::terrain` + the render path. The grass-block
    tint exclusion already matched (`terrain_particle_tint` returns
    None for `Tint::GrassBlock`).
  - 358 no-change (Potions): `Holder$Reference<Potion>` (26.1) vs
    `Holder<Potion>` (1.21.11) only — after normalizing that name
    the dumps are identical (same set, same effects). The Rust
    reads potions from the registry data.
  - 360 no-change (Climate): access flags only.
  - 361 CHANGED (worldgen features): RandomPatch / IceSpike /
    ForestRock implemented; EndSpike verified no-change; the 26.x
    `minecraft:spike` and `minecraft:block_blob` types removed.
    36 configured features use the five types at top level
    (flower 9, random_patch 24, no_bonemeal_flower 1, forest_rock
    1, ice_spike 1), plus 3 nested random_patch features inside
    forest_flowers' selector — all were silently SKIPPED before
    this port (the dispatch fallback rejects the types, and
    decoration counts them `unsupported`): no flowers, no random
    patches, no forest rocks, no ice spikes in 1.21.11 worldgen.
    - RandomPatchFeature (new file): a nested PlacedFeatureDefinition
      + tries/xz_spread/y_spread; the tries loop offsets by
      `nextInt(spread+1) - nextInt(spread+1)` per axis and places
      the inner feature (with its placement modifiers) at each
      offset, returning count > 0. All three ids (flower,
      no_bonemeal_flower, random_patch) decode to it — in 1.21.11
      all three registry entries are the same class. The catalog's
      `expand_feature` gains the three ids so a named inner placed
      feature resolves (mirroring root_system).
    - IceSpikeFeature (new file): descend through empty blocks to a
      snow-block footing; origin above by nextInt(4); height
      nextInt(4)+7; radius height/4+nextInt(2); 1/60 tall spike;
      tapered packed-ice cone (rim cells alone draw the random
      float, short-circuit preserved; mirrored below-layer write
      only when layer != 0 and r > 1); base skirt to y = 50 in
      runs, dropping past gaps. Replaceable: air/dirt/snow/ice
      (cone), plus packed ice (skirt); `isDirt` is the dirt tag.
    - ForestRock: `BlockBlobFeature.place()` was already the 1.21.11
      algorithm exactly, so only the decode differs — added
      `decode_forest_rock` (state only; the support check is the
      hardcoded dirt-or-stone tag predicate, decoded through the
      predicate codec; air is in neither tag, so no separate
      non-empty test is needed). The 26.x `minecraft:block_blob`
      arm + `decode` are gone (1.21.11 has no BLOCK_BLOB entry,
      no vanilla use).
    - EndSpike VERIFIED NO-CHANGE against the 1.21.11 SpikeFeature
      + SpikeCacheLoader decompiles: decode fields, the obsidian
      cylinder / guarded cage / crystal + bedrock + fire place, and
      the seed generation (`Util.toShuffledList` Fisher-Yates via
      `tb::shuffle`, `Mth.floor` centers, radius 2+size/3, height
      76+size*3, guarded size 1/2) all match end_spike.rs.
    - The 26.x `minecraft:spike` type is gone with it: dispatch arm
      + enum/error variants + evaluate arm + spike.rs deleted
      (1.21.11 has ICE_SPIKE, not SPIKE; no vanilla use).
    - SimpleBlockFeature half no-change: the only difference is
      `getOptionalState(level, random, pos)` (26.1, nullable) vs
      `getState(random, pos)` (1.21.11) — unobservable, since no
      vanilla 1.21.11 configured feature uses rule_based (the only
      provider that can return None without a fallback).
  - 362 no-change (structures): `HolderSet.empty()` vs
    `HolderSet.direct(new Holder[0])` (same empty set); 1.21.11
    adds a DataVersion <= 1493 guard returning CHUNK_LOAD_NEEDED
    instead of datafixing in place — the Rust has no 1493
    references and no partial-datafix chunk-lookup path.
    `ChunkPos.pack` -> `toLong`, `x()`/`z()` methods -> fields.
  - 363 CHANGED (special models): bell, book and end_cube are
    26.x-only (16 special types in 26.1, 13 in 1.21.11) — removed
    from SpecialKind (enum + decode arms + to_json) and from the
    item_render extents/submit matches, and the bell model layer
    dropped (the book layer stays: `submit_book` renders the GUI
    book via GuiBookModelRenderer, a different renderer). A pack
    model naming one now fails to decode, as in vanilla.
    1.21.11's STATIC_BLOCK_MAPPING/GIFT_CHEST needs no Rust
    counterpart: the Rust resolves specials through the item
    model's `special` node, and the Christmas chest texture is
    already handled.
  - 364 no-change (Options synthetics): requireNonNull ctors + the
    switch-map holder with the same three enums (InactivityFpsLimit,
    PrioritizeChunkUpdates, TextureFilteringMethod), initialized in
    a different order.
  - 365 no-change (login/connection): access flags + ctor noise
    only; server-side.
- Rank-2 A-batches 1-7: 70 items closed (16 changed, 54 no-change).
  work-items.json rank2.closed = 70 on 2026-11-19.
- A-batch 8 (366, 367, 368, 369, 370, 373, 374, 375, 376, 378),
  verified 2026-11-19, ported the same day:
  - 366 no-change (evoker/spellcaster goals): access flags +
    requireNonNull ctors + lambda method order. IllagerSpell has 6
    values in both versions. Server-side AI.
  - 367 no-change (NBT): ByteTag$Cache, one access-flag line.
  - 368 CHANGED (note block instruments): 26.1 has 27 instruments,
    1.21.11 has 23 — TRUMPET, TRUMPET_EXPOSED, TRUMPET_OXIDIZED and
    TRUMPET_WEATHERED (the copper-trumpet instruments, Type
    BASE_BLOCK) are 26.x-only; everything after PLING shifts down 4
    ordinals. `INSTRUMENTS` cut 27 -> 23 and `instrument_works_above`
    rethresholded 20 -> 16 (the mob-head instruments now start at
    16). With the 27-entry table a 1.21.11 zombie instrument (index
    16 in the data) read as "trumpet" and failed the works-above
    test, in both `note_block_instrument` and the play/trigger
    paths. A note block on copper now falls back to harp, as
    vanilla 1.21.11 does for unmapped base blocks. (WallBlock's
    `FluidState.is(WATER)` vs `getType() == WATER` is the same
    semantics; WallBlock$1 is switch-map order.) The separate
    8-entry INSTRUMENTS in fixes_old.rs is the pre-1.13 datafix
    table, untouched.
  - 369 no-change (SimpleContainer): 1.21.11 HAS a listeners list +
    addListener/removeListener, and `setChanged()` notifies them;
    26.1's `setChanged()` is an empty no-op. The Rust already
    models this through `ContainerHooks::changed` (`set` and
    `remove` both call it) — the 1.21.11 form.
  - 370 no-change (font): access flags + ctor order + `Matrix4fc`
    vs `Matrix4f` in renderSprite. Mojang font plumbing.
  - 373 no-change (camel husk): `interact(Player, Hand, Vec3)` vs
    `(Player, Hand)` (the 26.x signature gained the hit position);
    26.1 adds a `canFallInLove()` override returning false.
    `isBaby()` is false in both. Server-side.
  - 374 no-change (player): Abilities$Packed (same 7 components),
    Input, Player synthetics — access flags + ctor order only.
  - 375 no-change (MatrixUtil): `checkPropertyRaw` public vs
    private; `checkProperty` ORs cached properties in 26.1 vs
    re-determines in 1.21.11 (unobservable for a maintained
    matrix); `isOrthonormal` exists only in 1.21.11, but the Rust
    never checks orthonormality (precomputed quarter-turn tables).
  - 376 no-change (RenderType): 26.1-only `hasBlending()` /
    `outputTarget()`; invokevirtual vs invokeinterface for the GPU
    device/pass (the 26.x backend turned interfaces into classes).
    Mojang GPU plumbing.
  - 378 no-change (widgets): interface order; `extractWidgetRenderState`
    vs `renderWidget`, `item` -> `renderItem`, `fakeItem` ->
    `renderFakeItem` (the known render-API rename pattern).
- Rank-2 A-batches 1-8: 80 items closed (17 changed, 63 no-change).
  work-items.json rank2.closed = 80 on 2026-11-19.
- A-batch 9 (379, 380, 381, 382, 383, 384, 385, 386, 387, 391),
  verified 2026-11-19, ported the same day:
  - 379 CHANGED (llama decor): 1.21.11 renders the carpet decor on
    babies too (26.1 skips them); a trader llama is always
    `trader_llama` — 1.21.11 has no `trader_llama_baby` layer (26.1
    has both). `decor()` drops the baby exclusion and the
    trader-baby branch; the `llama_baby#decor` model stays.
  - 380 no-change (StructureManager): `ChunkPos.pack` ->
    `asLong`/`toLong`, `unpack` -> ctor, `containing` -> ctor —
    the known rename pattern, no semantic change.
  - 381 no-change (dolphin goals): the only real difference is the
    PlayWithItemsGoal MOVE flag, 26.1-only and server-side; the
    Rust's dolphin goals already match the 1.21.11 form.
  - 382 no-change (happy ghast controls): access flags + lambda
    renumbering only.
  - 383 CHANGED (PathType): 1.21.11 renames the fire/damage family
    and drops one value — FireInNeighbor -> DangerFire, Fire ->
    DamageFire, DamagingInNeighbor -> DangerOther, Damaging ->
    DamageOther, OnTopOfPowderSnow -> DangerPowderSnow,
    OnTopOfTrapdoor -> DangerTrapdoor; BigMobsCloseToDanger removed.
    PATH_TYPES 27 -> 26. Maluses verified from the jars:
    DANGER_FIRE 8.0, DAMAGE_FIRE 16.0, DANGER_OTHER 8.0,
    DAMAGE_OTHER -1.0, DANGER_POWDER_SNOW and DANGER_TRAPDOOR 0.0.
    1.21.11's mob-data.json uses DAMAGE_FIRE (547 states) +
    DAMAGE_OTHER (20) — with the 26.1 names those fell through to
    Blocked. Ported across 16 mobs files; `from_name` on the
    1.21.11 names; `path_type_of_mob`'s big-mob branch replaced
    with an `entity_width <= 1` guard + Open check.
  - 384 no-change (raider goals): access flags + lambda
    renumbering only. Server-side AI.
  - 385 no-change (DirectoryValidator): access flags +
    requireNonNull ctor pattern only.
  - 386 CHANGED (legacy noise wiring): vanilla 1.21.11's
    `RandomState` gates legacy init on
    `settings.useLegacyRandomSource()` — the nether's settings set
    it, the overworld's don't. Under legacy init, temperature and
    vegetation use the legacy nether-biome sampler with HARDCODED
    parameters (firstOctave -7, amplitudes [1.0, 1.0]) seeded from
    the world seed plus 0 and 1, recreated on every lookup; 26.x
    moved the legacy samplers to dedicated `nether/temperature` and
    `nether/vegetation` entries. Ported: `seeded.rs noise()` gates
    the legacy branch on `Factory::Legacy(_)` and matches
    `minecraft:temperature`/`minecraft:vegetation` with
    `create_parity(-7, &[1.0, 1.0])`; the `nether/*` matches
    removed. Two load-blocking gaps in the same path fixed:
    `density.rs` gains `shift_noise()` (the 1.21.11 shift JSONs
    carry the offset noise under `argument`, and the decoder
    required a `noise` field, so registry load failed), and
    `worldgen.rs decode_noise` now reads the old shape
    (`firstOctave`/`amplitudes` + `create_parity`) instead of the
    26.3 shape. The graph test and the `get_noise_sampler` doc
    updated to match. Follow-ups: the SHIFT -> zero rule only
    affects code-built routers; the port-pack upgrader still
    targets 26.3 noise shapes and needs its own item.
  - 387 no-change (SetComponentsFunction): MAP_CODEC -> CODEC, the
    codec field rename only; the wire shape is the same.
  - 391 no-change (worn-skull layer): CustomHeadLayer's skull
    branch IS really different (1.21.11 scales (1.1875, -1.1875,
    -1.1875), translates (-0.5, 0, -0.5), and calls submitSkull
    with a 180 deg rotation, whose body translates (0.5, 0, 0.5)
    and scales (-1, -1, 1); 26.1 scales uniformly and its
    submitSkull applies no transform), but it composes to exactly
    the uniform 1.1875 scale the Rust already applies: the two
    +/-0.5 translations cancel, and diag(-1.1875, 1.1875, -1.1875)
    composed with the head's 180 deg yRot is uniform 1.1875, with
    every skull head pivot on the Y axis. The placed-skull
    renderer already applies the 1.21.11 submitSkull transform.
    The two record classes are visibility only.
- A-batch 10 (394, 395, 396, 397, 398, 399, 400, 401, 402, 403),
  verified 2026-11-19, ported the same day:
  - 394 no-change (SystemReport): 26.1 has "Memory (heap)" and
    "Memory (non-head)" detail lines plus `printMemoryUsage`;
    1.21.11 drops both. The Rust's crash report is Horizon's own
    format — neither version's lines exist there.
  - 395 no-change (DedicatedServer$1, DedicatedServerProperties):
    requireNonNull ctor pattern, LOGGER private -> package-private,
    `acceptsTransfers` final -> non-final.
  - 396 no-change (VanillaPackResources): `getMetadataSection` IS
    structurally different (26.1 lazily loads and caches
    `resourceMetadata`, warns on parse failure; 1.21.11 re-reads
    `pack.mcmeta` per call and lets parse errors propagate), but
    there is no Rust surface — no `getMetadataSection` counterpart.
  - 397 no-change (Ocelot): `isTrusting` private -> package-private
    plus `is(Object)` -> `is(Block)`.
  - 398 no-change (RecipeManager, IngredientCollector,
    ServerDisplayInfo): access flags only.
  - 399 no-change (VaultBlockEntity$Server): `ItemInstance` (26.x)
    vs `ItemStack` (1.21.11) in the `resolveItemsToEject`
    signature; the body is identical.
  - 400 no-change (WorldBorder, Moving/StaticBorderExtent): fields
    private -> package-private; `SavedDataType` takes an
    `Identifier` (26.1) vs a raw String (1.21.11) — same id, codec,
    DataFixType; ctor field-init order swaps.
  - 401 CHANGED (BlockStateProvider): 1.21.11's
    `RuleBasedBlockStateProvider` is a standalone Record
    (`fallback` REQUIRED + `rules`, first match wins, else
    fallback), NOT a provider subtype and NOT in the 1.21.11
    registry (7 types; 26.1 adds `rule_based_state_provider`).
    It is used only by DiskConfiguration, and the bundle disk
    JSONs carry it with NO `type` field, which the Rust's typed
    decode rejected — the 4 disk features could not load. Ported:
    `Node::RuleBased` and the `minecraft:rule_based` arm removed;
    long-name arms added for all 7 vanilla types (the bundle uses
    `minecraft:simple_state_provider` etc., the decode accepted
    only short names); new `RuleBasedBlockStateProvider` record
    type with record-form decode and `get_state`; `disk.rs` decodes
    the record form and places unconditionally (verified against
    the 1.21.11 DiskFeature decompile). Supporting fix, pulled
    forward from item 111: `BlockStateJsonCodec::decode` now reads
    the NBT names — `full` form `Name`/`Properties` only (verified
    from the jar: `StateHolder.codec` dispatches on `"Name"`),
    compact form either naming — mirroring the 26.2 port; the
    26.1-base codec read `id` only and could not decode any vanilla
    `{"Name": ...}` state. Tests: 4 state_provider unit tests pass
    (record decode/eval, long names, record bounds 8192/8193 +
    shared budget) and the bounds integration test passes with the
    typed forms asserted rejected. Follow-ups: the client
    block_transform JSON parser still accepts `minecraft:rule_based`
    (Horizon content, no content uses it); the port-pack upgrader
    still maps `rule_based_state_provider` -> `rule_based` (its own
    item, as with the noise shapes).
  - 402 no-change (StructureTemplate palettes, JigsawBlockInfo,
    StructureBlockInfo): access flags plus `is(Object)` ->
    `is(Block)`.
  - 403 no-change (PerlinNoise): 1.21.11's `getValue` gains a 6th
    boolean arg (when true, each level's Y input is `-yo`, a
    horizontal slice), but the true path is DEAD in 1.21.11 — only
    `PerlinNoise.class` itself references the 6-arg descriptor in
    both jars, and its only call passes `false`. The live path is
    byte-identical to 26.1's 5-arg form, which the Rust's
    stack/smeared evaluation already implements.
- A-batch 11 (405, 407, 408, 409, 410, 411, 412, 413, 414, 415),
  verified 2026-11-19, ported the same day:
  - 405 no-change (IntegerProperty): empty diff, byte-identical.
  - 407 CHANGED (Husk): 26.1's Husk has its own BABY_DIMENSIONS
    (0.49 x 0.99, eye 0.825) plus a `getDefaultDimensions` override;
    1.21.11 has neither, so a baby husk inherits Zombie's
    (ZOMBIE 0.6 x 1.95 scaled 0.5, eye 0.93). Ported: the species
    table Husk entry `Baby::Abs(0.49, 0.98, 0.825)` ->
    `Baby::Abs(0.3, 0.975, 0.93)` (bit-exact under f32 halving).
    The triage listed this as quick — its changed-methods dict only
    compares methods present in both versions, so a whole method
    missing in 1.21.11 did not register (same blind spot as 396).
    The Zombie/Drowned/ZombieVillager entries carry the same 26.1
    values but belong to their own items.
  - 408 no-change (EntitySelectorOptions, ScoreboardSaveData):
    `entityTags()` -> `getTags()`; the `type` selector lambda goes
    from `entity.is(TagKey)` to `entity.getType().is(TagKey)` (the
    same check, inlined); `SavedDataType` ctor form, same id/codec.
  - 409 no-change (GossipContainer entries): private ->
    package-private only.
  - 410 no-change (TrackedWaypoint + Type): private ->
    package-private only.
  - 411 no-change (Guardian): `setMoving`/`setActiveAttackTarget`
    private -> package-private only.
  - 412 no-change (IpBanList, UserBanList): empty diff, byte-identical.
  - 413 no-change (SimpleBitStorage$InitializationException,
    Strategy): private -> package-private only.
  - 414 no-change (RandomSequences): `SavedDataType` ctor form plus
    the newer-javac `requireNonNull` on the synthetic outer-this
    param (26.1 only).
  - 415 no-change (TypedDataComponent$1): empty diff, byte-identical.
- A-batch 12 (416, 417, 418, 419, 421, 422, 423, 424, 425, 427),
  verified 2026-11-19, ported the same day:
  - 416 no-change (SignedMessageChain, LastSeenMessages,
    ProfilePublicKey$Data): private -> package-private;
    `LastSeenMessages.EMPTY` final -> non-final; the requireNonNull
    ctor pattern in the anonymous class.
  - 417 no-change (NeighborUpdater): 26.1 wraps the shape update in
    try/catch(Throwable) building a CrashReport ("Exception while
    updating neighbour shapes") and throwing ReportedException;
    1.21.11 lets it propagate. The Rust has no crash-report wrapper
    for shape updates — no surface. Rest is `is(Object)` ->
    `is(Block)` plus an access flag.
  - 418 no-change (LocationWrapper, PositionPredicate, PropertyMatcher):
    private -> package-private only.
  - 419 no-change (CriterionTriggerInstance): `validate` takes
    `ValidationContextSource` (26.1) vs `CriterionValidator`
    (1.21.11), but the Rust has no validation-context surface.
  - 421 no-change (TextureAtlasSprite): 26.1 adds
    `transparency()` delegating to `SpriteContents`; 1.21.11 has
    neither, and the Rust has no per-sprite transparency surface.
  - 422 no-change (ClientPackSource): `ResourceMetadata` (26.1) vs
    `BuiltInMetadata` (1.21.11) in the vanilla-pack metadata
    plumbing, plus a javac codegen difference; same area as 396,
    no Rust surface.
  - 423 no-change (TranslatableContents): `resolve` takes a
    `ResolutionContext` (26.1) vs `(CommandSourceStack, Entity)`
    (1.21.11) — the 26.x refactor of the same resolve logic; the
    Rust does not model component resolution at all.
  - 424 no-change (MobEffect AttributeTemplate, Details): private ->
    package-private only.
  - 425 no-change (SonicBoom): empty diff, byte-identical.
  - 427 CHANGED (PathNavigation): 26.1 gates `recomputePath` on
    `canUpdatePath()` (a mob that cannot update stays delayed and
    retries every tick); 1.21.11 recomputes whenever 20+ ticks
    elapsed, the gate living only inside `createPath` (verified from
    the jar: `canUpdatePath` is called by `createPath` and `tick`,
    not by `recomputePath`). Ported: `nav.rs recompute_path` drops
    the `|| !self.can_update_path()`; the `create_path_raw` and
    tick gates stay. `canCutCorner`'s FIRE_IN_NEIGHBOR /
    DAMAGING_IN_NEIGHBOR -> DANGER_FIRE / DANGER_OTHER renames were
    already ported in item 383.
- A-batch 13 (428, 429, 430, 431, 432, 433, 434, 435, 436, 437),
  verified 2026-11-19, ported the same day:
  - 428 no-change (Mannequin): 26.1's `aiStep` override (super +
    updateSwingTime) is gone in 1.21.11, and nothing in the 1.21.11
    mannequin's tick chain calls updateSwingTime (verified: no
    Mannequin/Avatar aiStep; `LivingEntity.aiStep`/`tick` never call
    it; only `Player.aiStep` and `Monster.aiStep` do) — but nothing
    in vanilla mechanics sets a mannequin swinging, so the Rust's
    shared `tick_swing` is a no-op for them either way. Follow-up:
    the Rust ticks swings for all mobs, vanilla 1.21.11 only for
    players + monsters — a shared-tick divergence for the tick items.
    `getProfile` public -> protected is Java-only.
  - 429 no-change (PrimedTnt$1): `is(Object)` -> `is(Block)` x2.
  - 430 CHANGED (ZombifiedPiglin): BABY_DIMENSIONS goes from 26.1's
    own 0.49 x 0.99 / eye 0.78 to 1.21.11's ZOMBIFIED_PIGLIN 0.6 x
    1.95 scaled 0.5, eye 0.97. Ported: the species table entry
    `Baby::Abs(0.49, 0.98, 0.78)` -> `Baby::Abs(0.3, 0.975, 0.97)`.
    The Piglin entry's identical values belong to the Piglin item.
  - 431 no-change (Blaze): PathType renames, already ported in 383;
    `setCharged` access flag.
  - 432 no-change (ChestType, Property$Value): 26.1's ChestType
    `CODEC` and Property$Value `valueName()` are gone in 1.21.11;
    the Rust handles chest_type as a string and has no `value_name`.
  - 433 no-change (WorldCarver): `is(Object)` -> `is(Block)` x6.
  - 434 no-change (WorldPresets): typeSwitch descriptor is a javac
    codegen difference, same 3-case switch; Bootstrap ctor flag.
  - 435 no-change (ChunkMap$DistanceManager): the requireNonNull
    outer-this pattern (26.1 only).
  - 436 no-change (PoiSection): ctor access flag.
  - 437 no-change (VibrationSystem): the ChunkPos rename pattern in
    `areAdjacentChunksTicking` (same 3x3 loop); CODEC final ->
    non-final plus access flags.
- A-batch 14 (438, 440, 441, 443, 444, 445, 446, 447, 448, 449),
  verified 2026-11-19, ported the same day:
  - 438 no-change (ChannelAccess, AbstractSoundInstance): access flags
    only plus the requireNonNull outer-this pattern (26.1 only).
  - 440 no-change (MapItemSavedData$MapDecorationLocation): record ctor
    private -> package-private.
  - 441 no-change (DebugLevelSource): the ChunkPos rename pattern
    (`x()`/`z()` -> record fields), same logic.
  - 443 no-change (MobRenderer): byte-identical after normalization.
  - 444 no-change (ServerOpList): byte-identical after normalization.
  - 445 no-change (WaypointTransmitter): ChunkPos `x()`/`z()` ->
    record fields in the view-distance check, same logic.
  - 446 CHANGED (CopperGolemOxidationLevels): the oxidation texture
    filenames differ — 26.1 `copper_golem_exposed/weathered/oxidized`
    vs 1.21.11 `exposed/weathered/oxidized_copper_golem` (verified
    from both jars' asset listings; the unaffected level is
    `copper_golem.png` in both). Ported: the three paths in
    `block_entity_render.rs` (the statue renderer reuses the entity
    texture files) + six in its tests renamed to the 1.21.11 forms.
    The mob renderer resolves textures through the prepared model /
    atlas system and has no hardcoded paths.
  - 447 no-change (WalkNodeEvaluator$1): PathType renames in the
    switch map (FIRE -> DAMAGE_FIRE, DAMAGING -> DAMAGE_OTHER),
    already ported in item 383.
  - 448 no-change (MoonBrightnessCheck): byte-identical after
    normalization.
  - 449 no-change (FeatureSorter$StepFeatureData): record ctor private
    -> package-private.
- A-batch 15 (450, 451, 452, 453, 454, 455, 456, 458, 459, 460),
  verified 2026-11-19, no code changes:
  - 450 no-change (MusicManager$MusicFrequency): `getNextSongDelay`
    private -> package-private.
  - 451 no-change (KeyframeTrackSampler$Segment): record fields/ctor
    private -> package-private.
  - 452 no-change (ScreenNarrationCollector$NarrationEntry,
    Screen$NarratableSearchResult): fields/ctor private ->
    package-private.
  - 453 no-change (WritableBookContent): byte-identical after
    normalization.
  - 454 no-change (TrapDoorBlock): `fluidState.is(Fluids.WATER)` vs
    `fluidState.getType() == Fluids.WATER` — equivalent:
    `TypedInstance.is(T)` is `typeHolder().value() == instance`.
  - 455 no-change (EntityCollisionContext): `ItemStack.is(Object)` ->
    `is(Item)`, the known overload rename.
  - 456 no-change (DimensionType$MonsterSettings):
    `IntProviders.codec(0, 15)` -> `IntProvider.codec(0, 15)` — same
    codec, same bounds; the Rust reads dimension_type from JSON.
  - 458 no-change (InterpolationHandler$InterpolationData): ctor
    private -> package-private.
  - 459 no-change (Biome$ClimateSettings): record fields/ctor private
    -> package-private.
  - 460 no-change (WorldgenRandom): `createThreadLocalInstance(seed)`
    vs `create(seed)` — both the same 48-bit LCG
    (SingleThreadedRandomSource vs LegacyRandomSource share
    multiplier/increment/mask and setSeed; verified from the jar);
    thread-safety only, same sequence per seed.
- A-batch 16 (461, 463, 464, 465, 467, 468, 469, 470, 471, 472),
  verified 2026-11-19, no code changes. Item 462 deferred as
  changed (see below); 472 pulled in to keep the batch at 10.
  - 461 no-change (ThrottlingChunkTaskDispatcher): `ChunkPos.unpack`
    -> `new ChunkPos(long)` in a debug lambda, same value.
  - 463 no-change (ModelPart): `Util.getRandom(cubes, random)`
    inlined as `cubes.get(random.nextInt(cubes.size()))` — identical
    (verified: that is exactly `Util.getRandom`'s body), same draw;
    Vertex u/v flags.
  - 464 no-change (PlayerModel): 1.21.11 has a `setAllVisible`
    override 26.1 lacks entirely, but it has no vanilla callers in
    either version and the Rust has no `set_all_visible` — no surface.
  - 465 no-change (ChatListener$Message): record ctor flag.
  - 467 no-change (SpriteSourceList$1): requireNonNull outer-this
    pattern (26.1 only).
  - 468 no-change (ArgumentUtils): typeSwitch descriptor is a javac
    codegen difference, same tableswitch (as item 434).
  - 469 no-change (VanillaRegistries): 26.1-only registries
    (PIG/COW/CHICKEN/CAT_SOUND_VARIANT, WORLD_CLOCK, VILLAGER_TRADE,
    TRADE_SET) have no Rust surface here — `sound_variants` covers
    wolves only (already 1.21.11-correct); world clocks belong to
    item 439; trade_set/villager_trade are data-driven datapack kinds.
  - 470 no-change (MultiPackResourceManager): catch type narrows
    `Exception` -> `IOException` — Java plumbing, no Rust surface.
  - 471 no-change (ContextMap): ctor flag.
  - 472 no-change (WeightedList$Compact): ctor flag.
  - 462 DEFERRED as changed (GuiEventListener): 26.1 adds
    `preeditUpdated(PreeditEvent)`; preedit is entirely 26.x
    (1.21.11 has no PreeditEvent, IMEPreeditOverlay, or preedit.png)
    while the Rust implements it (`preedit.rs`, `game/ime.rs`, call
    sites in game.rs, multiline_edit.rs, screen_draw.rs, ui.rs,
    sign_edit.rs). The triage missed it — its changed-methods dict
    only compares methods present in both versions (the 396/407
    blind spot). Removal belongs with the changed items.
- A-batch 17 (473, 474, 475, 476, 477, 478, 479, 480, 481, 482),
  verified 2026-11-19, no code changes:
  - 473 no-change (AttributeModifier): 26.1's `INTEGER_LIBRARY` is
    absent in 1.21.11, but the Rust's environment-attribute system
    has no integer kind (integer attributes are `Other`,
    override-only; `with_modifier` rejects modifiers on them).
  - 474 no-change (EnvironmentAttributeReader): 26.1's
    `getValue(LootContext, ...)` overload has no Rust surface — no
    loot code reads environment attributes.
  - 475 no-change (AttributeSupplier, AttributeInstance$Packed):
    record fields/ctor flags.
  - 476 no-change (TemptGoal): ctor flag.
  - 477 no-change (MushroomCow$Variant): `id` field + `byId` flags.
  - 478 no-change (GlowSquid): `is(Object)` -> `is(Block)`.
  - 479 no-change (AbstractPiglin): PathType renames, already ported
    in item 383.
  - 480 no-change (AbstractSkeleton$1): requireNonNull outer-this
    pattern (26.1 only).
  - 481 no-change (Spider): `is(Object)` -> `is(Block)`.
  - 482 no-change (ContainerEntity$1): ctor field-init order swap +
    requireNonNull pattern — javac codegen, same fields.
- A-batch 18 (484, 485, 486, 488, 489, 490, 491, 492, 493, 494),
  verified 2026-11-19, no code changes:
  - 484 no-change (PersistentEntitySectionManager$Callback): ctor
    flag + requireNonNull pattern + field-init order swap — javac
    codegen, same fields.
  - 485 no-change (GameEvent$ListenerInfo): byte-identical after
    normalization.
  - 486 no-change (FlatLevelGeneratorSettings): `is(Object)` ->
    `is(Block)`.
  - 488 no-change (LootContextParams): `TOOL` is
    `ContextKey<ItemInstance>` in 26.1 vs `ContextKey<ItemStack>`
    in 1.21.11, but the Rust models the loot tool as its own
    `NativeLootTool` (item + enchantments); 26.1's
    `ADDITIONAL_COST_COMPONENT_ALLOWED` is read only by the two
    enchant loot functions, whose Rust implementations never add a
    cost component — already 1.21.11-correct.
  - 489 no-change (PalettedPermutations): LOGGER flag.
  - 490 no-change (ClientHandshakePacketListenerImpl$State): enum
    field flags.
  - 491 no-change (LevelAccessor): 1.21.11 lists `LevelReader` in
    the extends clause — Java type hierarchy only.
  - 492 no-change (ServerLinks$Entry): record field flags.
  - 493 no-change (LanServerDetection): static field flags.
  - 494 no-change (TransportItemTarget): record field flags.
- A-batch 19 (495, 496, 497, 498, 499, 500, 501, 502),
  verified 2026-11-19, no code changes. This closes the last quick
  rank-2 items: of the 189 triage-EMPTY items, 188 are closed (item
  462 deferred as changed in A16). Rank 2 now stands at 188/492;
  the remaining 304 are changed items.
  - 495 no-change (Raid$RaiderType): enum field flags.
  - 496 no-change (NormalNoise$NoiseParameters): record field flags.
  - 497 no-change (Pack$Metadata): `description` flag.
  - 498 no-change (ServerRecipeBook$Packed): record field flags.
  - 499 no-change (Vindicator): field flags + the synthetic
    `access$000` moved in class-file order (same body).
  - 500 no-change (TrialSpawnerStateData$Packed): record field flags.
  - 501 no-change (BiomeFilter): CODEC final -> non-final.
  - 502 no-change (Score$Packed): record field flags.
- Rank-2 A-batches 1-19: 188 items closed (25 changed, 163 no-change).
  work-items.json rank2.closed = 188 on 2026-11-19.
- Item 462 (GuiEventListener), changed, ported 2026-11-19 as part of
  item 11's preedit removal: 26.1's `preeditUpdated(PreeditEvent)`
  default method has no 1.21.11 counterpart — preedit is entirely
  26.x (no `PreeditEvent`, `IMEPreeditOverlay` or `preedit.png` in
  the 1.21.11 client jar). Removed the Rust's preedit overlay
  (`horizon-client-render/src/preedit.rs`, the `report_caret` calls,
  `Game::ime_preedit`/`flush_ime`, the `Ime::Preedit`/`Disabled`
  arms and the key suppression); committed text still reaches the
  focused field through `ime_commit`, the 1.21.11 path.
  Rank 2 now 189/492 (26 changed, 163 no-change).
  work-items.json rank2.closed = 189 on 2026-11-19.
- Item 11 (game.rs: Minecraft, Gui, KeyboardHandler, ServerLevel,
  Block, BucketItem + 23 more classes), changed, ported 2026-11-19
  in parts (8fab09a, 048308d, 2e1d353, 91c3505; the preedit part
  landed with item 462 in bd11312):
  - Lightmap debug key (26.1-only F3+4): removed the
    DebugLightmapTexture action, keybind, debug chart + toggle —
    the only Options KeyMapping difference between the jars.
  - Chat gating: the 26.x ChatAbilities/ChatRestriction system is
    gone; 1.21.11 gates opening chat on ChatStatus (Hidden -> red
    `chat.disabled.options` overlay + narration, screen stays
    closed) and never gates sending once open. Removed the send
    gates (chat screen, native_send_chat, remote send_chat,
    send_unattended_command) and the restricted prompt.
  - Spectator click-to-spectate: 26.1 spectates client-side in
    startAttack; 1.21.11 spectates server-side
    (ServerPlayer.attack) and never swings as a spectator. The
    Rust now spectates the hit mob/painting via set_camera and
    never swings; the guest-side guard belongs to item 14.
  - Screens: ShareToLanScreen replaces WorldOptionsScreen
    (1.21.11 has no WorldOptions/Restrictions/MultiplayerOptions
    screens): game-mode cycle SURVIVAL/SPECTATOR/CREATIVE/
    ADVENTURE, allow-cheats toggle, port box with tryParsePort
    validation, pause-menu button gated to
    singleplayer-unpublished. Item 644 (ShareToLanScreen, rank 3)
    is already ported by this change.
  - No-change, verified against the jars: Gui.tick (one
    `is(Object)` -> `is(Item)` overload line), isBlocked (the
    Rust's social_hides matches 1.21.11's else-branch;
    shouldHideMessageFrom is byte-identical; the chat-status
    branch's effect is already produced by the receive-side
    visibility filter), startUseItem (merged vs split interact
    API, same behaviour), doWorldLoad, pick (moved to
    GameRenderer, identical), pauseIfInactive (inlined in
    GameRenderer.render, identical), close/destroy/disconnect/
    handleKeybinds/setScreen/shouldEntityAppearGlowing/
    setCameraEntity/tick/runTick (render-loop plumbing,
    relocations or the preedit removal), Block's 7 methods,
    BucketItem, NoteBlock, SpawnerBlock, TurtleEggBlock,
    TripWireBlock, FrogspawnBlock, ToastManager/TitleScreen/
    ParticleEngine extract renames, AbstractFurnaceScreen
    (numbers identical), PackConfirmScreen, ServerLevel
    weather/tick (identical algorithms; the level.dat split
    reversal is item 24, the world-clock removal is item 439).
  Rank 2 now 190/492 (27 changed, 163 no-change).
  work-items.json rank2.closed = 190 on 2026-11-19.
- Item 12 (game/creatures.rs), changed, ported 2026-11-19: 39 vanilla
  classes changed (195 methods, 49 named), javap diff 4,466 lines.
  Ported, all verified against the jars:
  - MushroomCow: the brown-mooshroom flower-feeding arm lost its
    adult gate (26.1's `isBaby()` check; 1.21.11 lets babies take the
    flower's effects).
  - Dolphin: `FollowPlayerRidden` -> `FollowBoat` — 1.21.11 adds one
    FollowBoatGoal at priority 8; 26.1 adds two
    FollowPlayerRiddenEntityGoal (boat + nautilus). Feed arm uses
    `is_baby()` instead of `can_age_up` (1.21.11 has no age locking).
  - Goat: `ramming_x_head_rot` flat 30.0 (26.1 gives babies 52.5);
    `age_boundary_reached` adds/removes horns (babies hornless,
    adults horned); newborns spawn hornless; `drop_horn`'s baby gate
    removed (babies have no horns to lose).
  - Sheep: `Sheep.ate` grows a baby on `is_baby()`, not `can_age_up`.
  - GlowSquidRenderer: babies use the adult model/texture (1.21.11
    has no `glow_squid_baby.png`).
  - GuardianRenderer: texture paths flattened (`entity/guardian`,
    `entity/guardian_elder`, `entity/guardian_beam`).
  - SpectralArrowRenderer / TippableArrowRenderer: `arrow_spectral` ->
    `spectral_arrow`, `arrow_tipped` -> `tipped_arrow`.
  - ThrownTridentRenderer / SpinAttackEffectLayer: `entity/trident/trident`
    -> `entity/trident`, `trident_riptide` likewise.
  - FlameFeatureRenderer: flame vertices at block light 15, sky light
    0 (1.21.11's hardcoded light 240); 26.1 keeps the entity's sky
    light.
  - IronGolem: doc comments updated `canAttack` -> `canAttackType`
    (behaviour already matched); DyeColor: doc comment cites
    `makeCraftColorInput` (implementation already 1.21.11's).
  - No-change, verified against the jars: TamableAnimal (26.1's
    5-arg `feed` is inlined in 1.21.11's Wolf.mobInteract, same heal),
    Chicken (the diff is the 26.x sound-variant system; the Rust is
    already 1.21.11-correct), Goat baby scale / powder-snow malus /
    brain form (no Rust surface; GoatAi is a separate item), Nautilus
    (brain form), SnowGolem / Creeper / Bogged / AmethystBlock /
    NetherPortalBlock / TargetBlock (overload/param noise), Projectile
    (`is(TagKey)` vs `getType().is(TagKey)`, equivalent),
    AbstractThrownPotion, Snowball, ThrownEgg (same odds; the variant
    application has the same outcome), EnderpearlItem / SnowballItem
    (shoot power 1.5 in both), SpawnEggItem (identical logic, static
    vs instance shape), Enchantment (`doPostPiercingAttack` vs
    `doLunge` is a pure rename — same body, component and callers;
    the FloatAction/Consumer shapes are equivalent),
    PlayerInteractTrigger (validation plumbing), the renderer
    `submit` / `submitModel` plumbing (CameraRenderState package move,
    RenderType param) and the llama-spit texture rename (no hardcoded
    Rust surface).
  - Referred: the full age-lock removal is item 23 (mobs/ai.rs);
    the goat's powder-snow malus is missing in the Rust (upstream
    gap, both versions).
  Rank 2 now 191/492 (28 changed, 163 no-change).
  work-items.json rank2.closed = 191 on 2026-11-19.
- Item 13 (realms.rs), changed, ported 2026-11-19: 73 vanilla classes
  changed (287 methods, 38 named), javap diff 8,760 lines, verified
  by whole-diff constants / method-call / branch-opcode comparisons
  plus direct reads of every flagged hunk. Ported:
  - RealmsWorldSlotButton: 1.21.11 truncates a slot name wider than
    64px to `plainSubstrByWidth(name, 64 - width("...")) + "..."`
    (`WORLD_NAME_MAX_WIDTH`, `DOTS`); 26.1 scrolled it in a
    `StringWidget`. New `clamp_text_dots` helper (vanilla's rule,
    reserving only the dots' width); other `clamp_text` call sites
    keep their `"... "` room.
  - RealmsSlotOptionsScreen: 1.21.11 is absolute-positioned
    (`row(i) = 40 + i * 13`, 170-wide columns at w/2 - 170 and
    w/2 + 10); 26.1 used a HeaderAndFooterLayout. Title at 17, the
    non-NORMAL warning label at 26 (red), name label centred over
    column 1 at 35, name field at 53, difficulty (col 2) at 53, game
    mode (col 1) and force game mode (col 2) at 79, spawn-protection
    slider (col 1) at 105, Done (`mco.configure.world.buttons.done`,
    not `gui.continue`) and Cancel at 209. Enable rules unchanged.
    Four sites updated: buttons, draw, text-field rect, slider click.
  - No-change, verified: RealmsMainScreen + all entries, the
    upload/download/backup/configure/players/template/reset screens
    (render -> extract, text/drawString/centeredText, addMessage ->
    setMessage renames; access flags; synthetic accessors; ctor
    field-init order; `isEscape()` vs `key() == 256`; `findByIndex`
    bounds-check vs exception — same behaviour); `scrollRate()` =
    entry height / 2 in both (the Rust already wheels at height/2);
    `scrollBarX` = row right + 8 in both; `forServiceError`,
    `isSelfOwnedServer`, `getAction`, `bindEntryToSelf` exist in both
    (access-flag false positives of the blind-spot script);
    `confirmDangerousOption` is dead code in 1.21.11 (private, zero
    callers); RealmsWorldOptions' `difficulty()` vs
    `difficultySettings().difficulty()` is API shape only (the Rust
    DTO carries the raw int); narration-only additions
    (`getNarrationMessage`, `addLabel`/`createLabelNarration` — only
    SlotOptions calls `addLabel`, drawn inline in the Rust).
  - Referred to item 24 (save.rs): `createTemporaryWorldFolder`
    writes level.dat with world gen + game rules INSIDE it in 1.21.11
    (`PrimaryLevelData.createTag`); 26.1 splits them into separate
    files. The Rust's `create_temporary_world` follows the 26.1
    split; drop its separate `write_world_gen_settings_with` call
    when item 24 reverses the split. The 26.1-only
    `updateSlotState(RealmsServer)` has no Rust surface (the Worlds
    tab redraws per-frame).
  Rank 2 now 192/492 (29 changed, 163 no-change).
  work-items.json rank2.closed = 192 on 2026-11-19.
- Item 14 (game/lan_actions.rs), changed, ported 2026-11-19: 8 vanilla
  classes changed (119 methods, 54 named), javap diff 6,257 lines,
  verified by whole-diff constants / method-call / branch-opcode
  comparisons plus direct reads of every hunk. Ported:
  - Player impulse context is Player-only in 1.21.11 (moved off
    LivingEntity): `setIgnoreFallDamageFromCurrentImpulse(boolean)`
    takes no position (grace 40 if true else 0); the position is set
    directly by `onExplosionHit` (always the player position, flag =
    wind charge) and `MaceItem.hurtEnemy` (calculateImpactPosition).
    `ImpulseContext` gained `ignore_fall_damage`, a 1-arg setter and
    `tick()` (Player.tick's grace countdown after updatePlayerPose);
    `cause_living_fall_damage` gates the min-distance on flag AND
    position; mob `impulse_grace_time` (field, tick-down, NBT
    save/load, armor-stand default) removed. `LivingFallWorld` gained
    `ignoring_fall_damage()`; the local player ticks in
    `simulation_tick_with_fluids`, loaded guests in
    `tick_guests_inner` (prediction calls are test-only, so no
    double-decrement).
  - Player save keys: `Player.addAdditionalSaveData` writes
    `current_explosion_impact_pos` (nullable),
    `ignore_fall_damage_from_current_explosion` (always) and the
    grace time; `LivingEntity` writes none. New `Level.player_impulse`
    (`ImpulseSave`, via `ImpulseSave::from_impulse`), set in
    `sync_level` and `playerdata::store`, saved/restored in
    `GuestSwap`; the writer drops a stale retained impact when a live
    body has none, and keeps the raw data with no live body. The
    projection test now covers ignore + impact in all ten phases
    (only "no body raw retention" keeps the stale values).
  - `guest_attack`: the border + `within_attack_range` guard now runs
    before the spectator check, and a spectator's click on a target
    sets the camera (`guest_set_camera`, 1.21.11 `ServerPlayer.attack`
    — no pickable check).
  - `UseItemOn`: no minY check at all; above maxY the build.tooHigh
    message is sent and the use skipped, but the block updates still
    run; spawn protection + border fold into one silent mayInteract
    check (no message, no 26.1 `build_limit_message(true, max_y)`
    quirk); a placement that did nothing sends build.tooHigh once,
    for an UP face at maxY only. `spawn_protection_message` removed
    (no callers left); `build_limit_message` is tooHigh-only.
  - Dig start: spawn protection + border merged into one silent
    mayInteract check (the ack puts the block back).
  - `within_attack_range`: 1.21.11 `entityAttackRange()` reads
    `getActiveItem` — the item in use while one is being used, else
    the main-hand item (was: always the acting hand).
  - No-change, verified: no `handleAttack` / `ServerboundAttackPacket`
    in the Rust (Interact dispatch already 1.21.11-shaped); game-rule
    packets are 26.1-only with no Rust surface; container click's
    `Input` enum is the ClickType ordinals; `sendOverlayMessage` vs
    `displayClientMessage(msg, true)` is the same wire behaviour;
    written-book resolve same outcome; PickItemFromEntity prints no
    profile in the Rust (already 1.21.11); swing duration already
    main-hand (mob swing hardcodes 6 — pre-existing gap both
    versions, item 23's file); `applyItemBlocking` identical bodies;
    Brain NBT shape identical (only the Java API differs — the save.rs
    comment now says `Brain.serializeStart`); player reach already
    3.0/4.5 and mobs have no `entity_interaction_range` in the Rust;
    lunge food gate already present; mayfly + FALL_ONE_CM already in
    `landing.rs`; propagate order unobservable; eating random already
    the level random; HoneyBlock `getRandom()` vs `random` same
    stream; BrewingStandMenu 0 methods changed; PermissionSet usages
    all in 26.1-only game-rule handlers; `isPickable` ->
    `canBeHitByProjectile`, `ItemStack.is(Object)` -> `is(Item)`,
    `$1`/`$2` switch-map renames and jump renumbering all noise;
    `dispense_fixture` sets only `impact_position`, matching the
    1.21.11 `VanillaDispenserOracle`.
  - Follow-ups, not this item: the Java probe oracles are still
    26.1-shaped (`VanillaLivingFallOracle`, `VanillaSceneMotionOracle`,
    `VanillaTeleportConsumeOracle` — LivingEntity impulse fields, 2-arg
    setter; they need Player-based actors to run against 1.21.11, and
    are not in the 17-harness CI gate); the Rust does not disconnect
    on an invalid-entity attack (1.21.11's `$1` handler does — noted
    gap in `guest_attack`).
  Rank 2 now 193/492 (30 changed, 163 no-change).
  work-items.json rank2.closed = 193 on 2026-11-19.
- Item 15 (game/remote.rs), changed, ported 2026-11-19: 20 vanilla
  classes changed (111 methods, 32 named), javap diff 3,836 lines,
  verified by whole-diff constants / method-call / branch-opcode
  comparisons plus direct jar reads (EntityType, EndFlashState,
  CauldronInteraction, EnchantmentHelper, ChatStatus, LevelAccessor)
  and the lunge.json data diff. Ported:
  - `BlockStatePredictionHandler` lost `onTeleport` and
    `lastTeleportSequence` in 1.21.11: `endPredictionsUpTo` always
    passes the prediction's player position to `syncBlockState`.
    Removed `Remote.last_teleport_sequence` (field, both inits, the
    handleMovePlayer set) and simplified `snap_target` to
    `(recorded, meets)` — `meets.then_some(recorded)` — with the
    snap test rewritten for the always-snap rule.
  - Squid and glow squid lost their baby override
    (`Squid.getDefaultDimensions` / `BABY_DIMENSIONS`): babies fall
    back to `LivingEntity.getDefaultDimensions`, type dims x
    `getAgeScale` (0.5). Both species defs changed from
    `Baby::Abs(0.5, 0.5, 0.37)` to `Baby::Scale(0.5, None)` — with
    the jar-verified 0.8 x 0.8 type dims (eye 0.4), babies are
    (0.4, 0.4, 0.2). No render change (the squid_baby model stays;
    the glow squid keeps one model/texture for both).
  - End flash: `ClientLevel.tick` calls
    `EndFlashState.tick(getGameTime())` in 1.21.11, not
    `tick(getDefaultClockTime())` — `tick_end_flash` now passes
    `world.game_time` instead of `world.day_time`.
  - Signed-command confirmation: `openSignedCommandSendConfirmationWindow`
    suggests into the chat box only when no screen is open AND the
    chat is allowed (`getChatStatus().isChatAllowed(isLocalServer())`:
    Hidden-by-options means not allowed; the launcher/profile
    statuses need inputs this client does not have). The Rust never
    had the gate — `send_unattended_command` now offers Copy instead
    of Suggest when the chat is Hidden.
  - Comment-only: `handleContainerInput` -> `handleInventoryMouseClick`
    and `ContainerInput` -> `ClickType` (remote.rs, remote_stations.rs,
    tick_script.rs, container_screen.rs, menu.rs, lan/hub.rs,
    horizon-protocol play.rs — the Rust's `Input` enum is already the
    ClickType ordinals); `signed_chat_line` cites `ChatComponent.addMessage`
    (1.21.11's single entry); cauldron.rs cites `InteractionMap`;
    enchant_effects.rs cites `doLungeEffects` (the
    `minecraft:post_piercing_attack` data key is unchanged) and the
    lunge gate's mayfly form (spectators never jab, so may-fly is
    creative here); spear.rs cites `lungeForwardMaybe`;
    shader_packs.rs cites the internal default-clock model (1.21.11
    has no `getDefaultClockTime`; `getGameTime` is a `LevelAccessor`
    default).
  - No-change, verified: `getShade` already matches 1.21.11 exactly
    (`lighting::cardinal`); `getBlockTint` averaging identical (the
    `& 255` masks are no-ops on averages); SetTime/clocks/tickTime
    owned by the translate layer (round-trip tested); chat gating
    already at the ChatComponent level; SystemChat overlay already
    routed; disguised chat already tag-less Secure; attack already
    sends ATTACK with the sneak flag for all modes, local effects
    only when not spectator; interact already sends INTERACT_AT then
    INTERACT; demo mode has no Rust surface; game-rule packets are
    26.1-only; `handleUpdateTags` has no client handler; difficulty
    change has no reaction; `lungeForwardMaybe` is a client no-op
    (LivingEntity's body runs only on ServerLevel; Player's override
    gates the same call on food/mayfly; the lunge.json data diff
    shows only the food term moving from data to the Player gate,
    and the spectator difference is unreachable — spectators never
    jab); cauldron `InteractionMap` is item-keyed with a
    TRY_WITH_EMPTY_HAND default (same as 26.1's DEFAULT), and the
    dyeable tag's members are exactly the items 1.21.11 registers
    for dye removal; Interaction entity handling never used the
    dropped Vec3; merchant screen already 276x166 with the tooltip
    drawn last; `ClientLevel.update()` (light) has no Rust
    equivalent (light runs on chunk changes); `prepareWeather` is a
    no-op for a fresh client level (isRaining false); marker
    particles never use a random for the marker spawn itself, and
    the animateTick position sampling is unseeded in both versions;
    leaf tint already resolves through the pack colormap/default;
    no client `handleLowDiskSpaceWarning` surface; `setPermissions`
    has no gamemaster reaction; `ClientPacketListener.getId()` has
    no Rust surface; `ClientChunkCache` hunks are accessor noise;
    `tickDayTime` defaults false until the first SetTime packet, but
    the server sends SetTime at login, so the pre-first-packet
    window is a few ticks at most (translate-layer concern, no
    change to the 1.0 clock-rate default).
  Rank 2 now 194/492 (31 changed, 163 no-change).
  work-items.json rank2.closed = 194 on 2026-11-19.
- Item 16 (block_entity_render.rs), changed, ported 2026-11-19: 33 vanilla
  classes changed (219 methods, 20 named), javap diff 8,286 lines,
  verified by whole-diff constants / method-call comparisons plus
  direct jar reads (SignRenderer, SkullModel, DragonHeadModel,
  LightCoordsUtil, TheEndPortalBlockEntity, SignBlock subclasses).
  Ported:
  - Four texture-path moves, jar-verified in the 1.21.11 client jar
    (10 sites): `entity/end_portal/end_portal` -> `entity/end_portal`,
    `entity/beacon/beacon_beam` -> `entity/beacon_beam`,
    `entity/end_portal/end_gateway_beam` -> `entity/end_gateway_beam`,
    `entity/enchantment/enchanting_table_book` ->
    `entity/enchanting_table_book`.
  - Chest Christmas precedence: 1.21.11's `getChestMaterial` checks
    xmas before copper (26.1 checked copper first), so copper chests
    wear the christmas textures on 24-26 December and only ender
    chests keep theirs. Both xmas sites now match the four copper
    chest blocks (waxed variants are unwaxed by `classify` first).
  - Comment-only: sign text cites `SignRenderer.translateSign` /
    `HangingSignRenderer.translateBase` + `translateSignText` (the
    unified `SignRenderer` replaces `StandingSignRenderer`); end
    portal cites `getOffsetDown/getOffsetUp` and `renderCube`/
    `renderFace`; lectern cites `bookState = new State(0, 0.1, 0.9,
    1.2)`; `BookModel.State` cites the (animationPos, pageFlip1,
    pageFlip2, open) record.
  - Verified no-ports: sign transforms identical in every value
    (2/3 scale, hanging text 0.9, offsets (0, 1/3, 0.046666667) /
    (0, -0.32, 0.073), wall (0, -0.3125, -0.4375); rotation via
    `convertToDegrees`/`toYRot()` in both versions) — only the
    structure changed (precomputed `SignTransformations` ->
    `translateSign`/`translateSignText`); end-portal caps identical
    (0.375/0.75, `shouldRenderFace` Y-only in both, so the
    full-height side faces in 1.21.11's `renderCube` are never drawn
    for portals) and the winding preserved; book openness formula
    moved into `setupAnim` with the same values (the Rust already
    computes it); copper golem y-rot move is a no-op by the
    Rz(PI) conjugation; skull rotation moved from the PoseStack into
    the model state, identical by the (-1,-1,1) conjugation, and the
    `entityCutoutNoCullZOffset` switch matches the Rust's already
    two-sided entity pipeline; chest transform and lid easing
    (`1-(1-open)^3`) identical; `BrightnessCombiner` is an exact
    inline of `LightCoordsUtil.max`; conduit constants/calls
    identical (only the `CondiutRenderState` typo rename); shulker
    `prepareModel` repeats the 26.1 matrix sequence exactly; enchant
    table and lectern book states carry the same four values;
    `LayerDefinitions` only reorders plus the Adult/Baby model merge
    (other items) and the moved, byte-identical `createSignLayer`;
    font/glyph classes, `BellModel$State`, `TextFeatureRenderer` and
    `DoubleBlockCombiner` are access/rename/overload noise.
  Rank 2 now 195/492 (32 changed, 163 no-change).
  work-items.json rank2.closed = 195 on 2026-11-19.
- CI run 44538822133 (items 11+462+12+13 milestone, on fcfc909)
  SUCCESS 2026-11-19, profile unchanged: 5 pass of 23
  (block_properties, component_patch, food, lan, resource_pack),
  18 fail — every harness status identical to run 44518329820, only
  host/timing/content-key fields differ. D1 fails on the same 26 dead
  production tags; D2 passes (0/0, 440 protocol tests); D3 fails
  (rank-2 303 open at the pin snapshot); D4/D5 blocked on the stamp
  quirk; D6 deferred. Run 44536718386 (same milestone, on b458e19)
  FAILED on a transient Maven Central 404 (asm 9.9.1 jar) and was
  superseded by this re-dispatch, not recorded.
- Item 17 (chat.rs, 24 classes, 152 methods, 26 named) CLOSED
  2026-11-19, changed. 1.21.11 has no add-time chat-visibility
  filtering: `ChatAbilities` / `ChatRestriction` / `GuiMessageSource`
  are gone, `ChatListener.showMessageToPlayer` checks only trust /
  blocked / fully-filtered and `handleSystemMessage` only
  hideMatchedNames / blocked, and `chatVisibility` is read only by
  `getChatStatus` (send gate), `isChatHidden` (render gate) and the
  options screen. Ported: `ChatAbilities` deleted outright; `Chat`
  keeps just `visibility: ChatVisibility`, `set_abilities` →
  `set_visibility` (no rewrap); `push_message` always adds + logs;
  `rewrap` always wraps all messages; the `shows` gates in
  `signed_chat_line` and `local_chat_line` removed (the first also
  wrongly suppressed narration, `mark_message_processed` and the
  chat-delay reset); `layout` returns empty when Hidden
  (`isChatHidden`). `ChatSource` stays for the compact-chat dedup.
  Tests rewritten (`the_chat_option_never_filters_the_lines`,
  `a_guests_line_joins_at_every_visibility`); 28/28 chat tests pass.
  Comment-only: bed-chat submit re-cited to
  `InBedChatScreen.keyPressed` (`closeOnSubmit` is gone; the bed
  screen intercepts Enter itself), `clipText` citation fixed (method
  removed in 1.21.11), `extractRenderState` → `render`,
  `addServerSystemMessage` → `addMessage(Component)`. Verified
  no-ports: `GuiMessage`/`Line`/`Tag` moved to `net/minecraft/client/`
  (source field dropped, `Line` denormalized to addedTime + tag),
  render math identical, restricted-chat machinery removed (the Rust
  never had it), suggestion restriction machinery already removed in
  item 11, usage offset 14+13=27 and suggestion-list rect identical,
  `QUEUE_EXPAND_ID` in both versions (the Rust has no queue display —
  pre-existing gap), `FilterMask` access noise, `clipText` removed,
  `AbstractCommandBlockEditScreen` render renames only.
  Rank 2 now 196/492 (33 changed, 163 no-change).
  work-items.json rank2.closed = 196 on 2026-11-19.
- Item 18 (camera.rs, 7 classes, 94 methods, 28 named) CLOSED
  2026-11-19: no behaviour change — comment-only. 1.21.11 guts `Camera`
  (85 → 46 members) and moves the FOV/projection machinery to
  `GameRenderer`, but every computation camera.rs mirrors is identical:
  `tickFov` byte-identical (moved Camera → GameRenderer);
  `GameRenderer.getFov(camera, partial, useFov)` merges calculateFov /
  calculateHudFov / modifyFovBasedOnDeathOrFluid with the same ops and
  constants (panoramic 90, hud base 70, death /(1+2(1−500/(500+min))),
  fluid ×lerp(scale,1,0.85714287)); `Camera.setup` == `alignWithEntity`
  (detached/mirrored now params, not read from options); `getMaxZoom` /
  `move` / `setRotation` identical; `getNearPlane()` == `getNearPlane(fov
  option)` (window aspect, fov option, (double)0.05f — Rust matches
  exactly incl. widening order); `getFluidInCamera`, bobView/bobHurt,
  `getFieldOfViewModifier`, `belowNameDisplay` (moved Entity →
  AbstractClientPlayer, byte-identical), `getDepthFar` =
  max(renderDistance×4, cloudRange×16), culling projection
  max(fov, fovOption) + perspective(0.05, depthFar), view rotation
  `Matrix4f.rotation(rotation().conjugate())` (now inline in renderLevel /
  projectPointToScreen), `projectPointToScreen` /
  `projectHorizonToScreen` (tan ratio, ±90° → ±∞), boss darkening tick
  (0.05 up / 0.0125 down, field renamed `darkenWorldAmount`), HUD
  projection 0.05→100 at hudFov — all identical. `CameraRenderState` is
  a tiny struct in 1.21.11 (blockPos, pos, initialized, entityPos,
  orientation; no entityRenderState/hudFov/depthFar/projection). Debug
  crosshair is `DebugScreenOverlay.render3dCrosshair` in both versions
  (`render3dHud` and `DebugCrosshairRenderer` exist in neither).
  Two Rust doc claims corrected (wrong in both versions): vanilla passes
  its projection planes straight (the reversal is the Rust's own
  reversed-Z pipeline), and `Frustum.getNearPlaneBounds` does not exist
  (near_plane_bounds is the Rust's own construction). Pre-existing
  divergences noted, not fixed: sleeping camera moves 0.15 up in the
  Rust vs 0.3 in both vanilla versions; the Rust's stairs/slabs
  near-plane view-blocking test matches neither version (vanilla uses
  the 2-arg `isViewBlocking` = suffocating predicate, no near-plane box
  in the call chain); hand projection far is depth_far in the Rust vs
  100.0 in vanilla. Referrals: item 42 (gfx.rs hudFov / Projection /
  render3dHud cites), 157 (fishing.rs getNearPlane(fov)), 410
  (locator_bar.rs projectHorizonToScreen), 443 (names.rs
  belowNameDisplay), 621 (render_tick.rs view-blocking cite + the
  near-plane divergence), 753 (projection buffer classes).
  Rank 2 now 197/492 (33 changed, 164 no-change).
  work-items.json rank2.closed = 197 on 2026-11-19.
- Item 19 (mobs/piglin.rs, 11 classes, 99 methods, 26 named) CLOSED
  2026-11-19: CHANGED — baby piglin ears 5° → 30°. 26.1's
  `AbstractPiglinModel.setupAnim` read the default ear angle from the
  virtual `getDefaultEarAngleInDegrees()` (adults 30°, babies 5°, via
  separate Adult/Baby model classes); 1.21.11 deletes the four
  subclasses, makes `AbstractPiglinModel` concrete, and uses the literal
  0.5235988f for every piglin. `ears()` now always uses 0.5235988
  (bit-equal to 30° × DEG_TO_RAD); new test pins ±0.60359877 at rest for
  baby and adult. Verified no-ports: hoglin `createBabyLayer` mane flows
  through prepare (Rust already selects `hoglin_baby#main`); the baby
  head-lift `head.y += f * 2.5` is new in 1.21.11 but the Rust already
  has it; piglin clothing parts (sleeves/pants/jacket) are data-driven
  geometry; `INTERACTABLE_DOORS` is a new memory no piglin code reads;
  the sensor inlines `findNearbyAdultPiglins` unchanged; Zoglin/HoglinAi/
  PiglinAi/PiglinBruteAi activity contents identical (brain-provider API
  refactor only); `SpearAttack`'s new `approachDistanceSq` is stored but
  never read (Rust's `PiglinSpear` already carries both squared
  distances); `wantsToDance` switched RNGs (Xoroshiro → LegacyRandom)
  but the Rust already uses LegacyRandom; `broadcastRetreat`'s
  `instanceof Piglin` check moved from the consumer into a filter (Rust
  already skips brutes); `MAX_TIME_BETWEEN_HUNTS` was dead in 26.1.
  Rank 2 now 198/492 (34 changed, 164 no-change).
  work-items.json rank2.closed = 198 on 2026-11-19.
- Item 20 (mobs/villager_brain.rs, 32 classes, 148 methods, 18 named) CLOSED
  2026-11-19: CHANGED — baby villager dims, finalizeSpawn typing gate,
  VillagerDataFinalized write dropped. 1.21.11 deletes
  `Villager.getDefaultDimensions` (BABY_DIMENSIONS 0.49 x 0.99, eyes 0.63),
  so a baby takes the type's box at `getAgeScale` 0.5: 0.3 x 0.975, eyes
  0.81 (`Baby::Abs` -> `Baby::Scale(0.5, None)`, new dims test).
  `Villager.finalizeSpawn` types by biome only for command, spawn egg,
  spawner or dispenser — the `DATA_VILLAGER_DATA_FINALIZED` flag is gone
  and structure villagers keep their template's type (verified in the
  desert `unemployed.nbt`); the Rust's `StructureMobFinalizer` was
  overwriting it. Zombie villagers get the mirror rule
  (`REASONS_NOT_TO_SET_TYPE`); both reason sets pinned by a test.
  `Villager` no longer writes `VillagerDataFinalized` (entity_nbt write
  removed; the zombie-villager save/cure half is item 664's).
  `MemorySlot` doc cites updated to `ExpirableValue`. Verified no-ports:
  the Brain core loop and every memory accessor are semantically
  identical across the storage refactor (expiry identical; the empty-
  memory `getTimeUntilExpiry` MAX -> 0 change is unobservable — every
  `.ttl()` caller is value-guarded); brain serialization already matches
  the 1.21.11 codec (`value` + lenient optional `ttl`); activity lists
  identical (provider supplier -> explicit `addActivity`); goal packages
  identical (profession param never loaded); `pickNextTarget` reservoir
  sampling identical; `startSleeping` moved its LAST_SLEPT / walk-target
  work out of `SleepInBed` with no net change; `shouldRestock` day count
  is the same value via `getDayCount`; `getBreedOffspring`'s added
  finalizeSpawn(BREEDING) is a no-op for the Rust's birth path.
  Referrals: item 129 (updateTrades -> hardcoded `VillagerTrades.TRADES`
  + TRADE_REBALANCE flag; the class is new in 1.21.11), 751 (the
  `VillagerTrades$*` listings), 664 (zombie-villager save/cure +
  `data_finalized` removal), datafix (`VillagerDataFinalized` in
  fixes_c.rs).
  Rank 2 now 199/492 (35 changed, 164 no-change).
  work-items.json rank2.closed = 199 on 2026-11-19.
- Item 21 (title.rs, 26 classes) CLOSED
  2026-11-19: CHANGED — Esc closes the Optimize screen; the file-fixing
  system is gone (dead UI plumbing removed); the restore wait shows
  "Attempting to restore world...". 26.1's
  `OptimizeWorldScreen.shouldCloseOnEsc` returned false; 1.21.11 removes
  the override, so Esc closes it (`removed()`/`tick()` identical) — the
  Rust's `escape()` now runs the Cancel path, with a new test
  (`escape_stops_the_optimization`). 1.21.11 deletes FileFixerUpper, the
  `UpgradeStatusTranslator`, `LevelSummary.requiresFileFixing`,
  `UPGRADE_AND_PLAY_WORLD` and `BackupStatus.FILE_FIXING_REQUIRED`, so
  `WorldEntry.file_fixing`, the "Upgrade and Play" label, the
  `requiresFileFixingTooltip.*` tail, the open-flow file-fixing prompt,
  `begin_optimize`'s gate arm, `save::needs_file_fixes` and
  `datafix::files::requires_file_fixing` are all removed; `can_edit` is
  `!corrupted && !disabled()` and the backup prompt's `(None, None)` arms
  are `unreachable!`. The stale file-fixing test is removed (broken since
  rank-1 item 9 stubbed the gate). `makeBackupAndShowToast` is synchronous
  in 1.21.11 with no waiting screen; `attemptRestore` shows
  GenericMessageScreen("recover_world.restoring") — the Rust keeps its
  async worker + wait deliberately, but shows the restoring text for a
  restore. Comment-only: status texts cite `WorldUpgrader`'s statics
  (same keys); the render renames (`extractRenderState` -> `render`,
  `renderLogo`, `renderContent`, `renderContents`,
  `renderMenuBackground`); `WorldCreationGameRulesScreen` ->
  `EditGameRulesScreen`; `getUnfixedDataTagWithFallback` ->
  `getDataTagFallback`; `collectIssues` -> `RecoverWorldDataScreen.collectIssue`.
  Verified no-ports: `attemptRestore`'s pre-check/fail path identical (the
  Rust's `attempt_restore` matches 1.21.11 exactly);
  `PresetFlatWorldScreen.save`, `openDataPackSelectionScreen`,
  `openExperimentsScreen`, `updateButtonValidity`,
  `PresetsList$Entry.select` byte-identical (they only lost `private` —
  method.py misses package-private methods); the DEBUG create branch is
  the same behaviour in both versions (the Rust's missing debug
  special-casing is a pre-existing gap, not fixed here); `createInfo`
  identical; `BackupConfirmScreen.init`/`keyPressed`,
  `ConfirmExperimentalFeaturesScreen.init`, `tryApplyNewDataPacks`,
  `blitSlot`/`getDisplayItem` identical. Referrals: item 24 (level.dat
  split reversal, `createFromExisting` game-rules storage), items 292/609/755
  (game-rules screens -> unified `EditGameRulesScreen`).
  Rank 2 now 200/492 (36 changed, 164 no-change).
  work-items.json rank2.closed = 200 on 2026-11-19.
- Item 22 (native_hooks.rs, 11 classes) CLOSED
  2026-11-20: CHANGED — vegetation stands on dirt-or-farmland; stems stand
  on farmland only and fruit on farmland-or-dirt; the screen narration hint
  is said before the widget. 1.21.11 deletes `BlockTags.SUPPORTS_VEGETATION`
  and `VegetationBlock.mayPlaceOn` becomes dirt-tag-or-farmland (the dirt
  tag holds the same blocks the old tag did, farmland aside) — the Sim's
  `Class::Vegetation` and the small-dripleaf water fallback follow it, and
  the registry gains a `SurvivalRule::Vegetation` backed by a dirt ∪
  farmland set (the old `TagBelow:minecraft:supports_vegetation` was a dead
  tag literal). 1.21.11 deletes `StemBlock`'s `stemSupportBlocks` /
  `fruitSupportBlocks` tag fields and hardcodes farmland-only placement and
  farmland-or-dirt fruiting, so the Sim splits the stem arm off the tag
  check, `stem_random_tick` checks dirt ∪ farmland, and the registry gains
  a `BlockBelow` rule form that prepare binds stems to (the form prepare
  had flagged as missing). `SmallDripleafBlock`'s tag was renamed
  `small_dripleaf_placeable` (same contents) — drive-by rename, the class
  has no report item. `Screen.updateNarrationState` narrates the usage hint
  after the title whenever the screen narrates navigation, before the
  widget (its no-widget branch is deleted) — `screen_narrator.rs` mirrors
  the move; four narrator tests updated to the new output. New Sim tests
  (fixture-gated): vegetation on dirt/farmland, stem on farmland only,
  small dripleaf on clay/watered dirt. Verified no-ports:
  `ServerPlayerGameMode` (a redundant spawn-protection check removed;
  `mayInteract` still gates, and the Rust never sent the message),
  `PlayerList` (the `load` change is item 24's level.dat reversal; the
  join-time packet is the clock removal, handled by the adapter),
  `FireBlock` (only the golden-dandelion odds go — data-driven via
  prepare), `CampfireBlock` (equivalent `getType()` forms), `Holder`
  (the Rust's `Holder` is the attachments enum; `direct(T)` survives),
  `AnvilMenu` (overload resolution), `EntitySelectorOptions$Option` /
  `DataValue` (access only). Referrals: item 24 (player-tag load), item 27
  (DoublePlantBlock), item 757 (FungusBlock/RootsBlock rename + hardcoded
  ground), items 87/519 (`DebugScreenOverlay.extractRenderState` cite).
  Rank 2 now 201/492 (37 changed, 164 no-change).
  work-items.json rank2.closed = 201 on 2026-11-20.
- Item 23 (mobs/ai.rs, 7 classes) CLOSED
  2026-11-20: CHANGED — the age lock is gone, and with it every ageable
  species' entity-data index shifts down by one. 1.21.11 deletes the whole
  age-lock mechanic (`AGE_LOCKED`, `setAgeLocked`, `canAgeUp`,
  `isAgeLocked`, the `AgeLocked` NBT key, the golden dandelion, the
  `#cannot_be_age_locked` tag), so the Rust deletes `Extra.age_locked` /
  `age_lock_particle_timer`, `can_age_up`, `set_age_locked_data`,
  `tick_age_lock_timer`, `age_lock_takes`, `set_age_locked`, the
  golden-dandelion interact branch and the golden-dandelion stew effect,
  and mod API hook 282 (`native_player_toggle_entity_age_lock`, now a
  gap). The mob-AI / ageable-load / command-exec harness drivers and both
  Java oracles drop their age-lock fixtures (the squid oracle's
  `isAgeLocked` override would not compile against 1.21.11). AgeableMob
  keeps only DATA_BABY_ID (16), so every accessor of every ageable species
  shifts down exactly 1 — verified by counting `defineId` calls per class
  hierarchy in both jars — and both the send side (`lan.rs`) and the
  receive side (`remote_mob_data.rs`) shift together (Panda 17–22, Wolf
  interested 19 / collar 20 / anger 21 / variant 22, Cat variant 19 /
  lying 20 / relaxing 21 / collar 22, Llama strength 19 / variant 20,
  Villager data 18, and the rest); Piglin, Zombie and all non-ageable
  species do not shift. The client-invented llama/donkey/mule "chested"
  entry moves 19→18 to keep its spare slot. Slime size at 18 is a
  PRE-EXISTING BUG (vanilla has it at 16 in both versions) — noted, not
  fixed. `Mob.requiresCustomPersistence` drops the leash clause: leashed
  mobs can despawn in 1.21.11 (new test). MagmaBlock gains bubble-column
  maintenance (`onPlace` / `updateShape` schedule a tick in 20; `tick`
  runs `BubbleColumnBlock.updateColumn` above) — the Sim gains the three
  arms; SoulSandBlock (item 68) has the identical trio. Verified no-ports:
  `Mob` target validation (`asValidTarget` gone; the per-tick drop mirrors
  `TargetingConditions`), HappyGhast/Hoglin brain-provider refactor
  (content is items 38/19), NaturalSpawner, Creaking path navigation.
  Referrals: item 47 (NeutralMob Peaceful removal + Ghast anger), item 48
  (`TargetingConditions` + `canAttackType`), item 58 (lunge), item 68
  (soul sand), items 97/178 (water side), item 24 (save.rs).
  Rank 2 now 202/492 (38 changed, 164 no-change).
  work-items.json rank2.closed = 202 on 2026-11-20.
- Items 24–27 CLOSED 2026-11-20 (no PORT-NOTES entries were written for
  them; their notes are the untracked workdir files rank2-item-24.md …
  rank2-item-27.md + TRIAGE files): item 24 (save.rs — the level.dat split
  reversal), item 25 (mobs/render.rs — drop the 26.x baby textures, restore
  the 1.21.11 variant names, render equipment on babies), item 26
  (motion/fluids.rs — the 1.21.11 fluid update shape), item 27
  (level/bonemeal.rs — drop the 26.x build-height guards, farmland soil
  checks, flower features). All four CHANGED. Rank 2 was 206/492
  (42 changed, 164 no-change) before item 28.
- Item 28 (particles.rs, 36 classes) CLOSED
  2026-11-20: CHANGED — four particles' light is the constant 240 in
  1.21.11, and GlowParticle renders translucent. `FlyStraightTowardsParticle`
  (ominous spawning), `ShriekParticle`, `TrialSpawnerDetectionParticle` and
  `VibrationSignalParticle` return 240 (sky 0, block 15) from
  `getLightColor`; 26.x replaced it with `withBlock(super, 15)`, which keeps
  the sky light — the four Rust arms move from `Light::Block15` to the
  existing `Light::Bright240` (VaultConnection and SculkCharge already used
  it). `GlowParticle.getLayer()` is TRANSLUCENT in 1.21.11 (OPAQUE in 26.1),
  so Glow, WaxOn, WaxOff, ElectricSpark and Scrape join
  `translucent_layer()` (item 192's file; item 28's mechanism). Glow's
  light math is unchanged — 1.21.11 inlines `addSmoothBlockEmission`
  (verified identical: smoothBlock/smoothSky are raw byte extracts and
  15*16 = 240) — so the Glow arm keeps `Light::AgeEmission`.
  `SimpleVerticalParticle` + the pause/reset-mob-growth providers are gone
  in 1.21.11 (the age-lock particles; item 23 removed the mechanic) and
  nothing in the Rust references them. `SingleQuadParticle$Layer` drops
  from 6 layers to 4 and loses `bySprite()`; the observable effect is
  TerrainParticle's constructor (always-translucent TERRAIN/ITEMS) — that
  is item 354, referred. No-change verdicts, all jar-verified:
  LevelEventHandler (same 79 cases; ItemParticleOption takes an ItemStack
  in 1.21.11 vs an Item in 26.1, same bursts), ParticleUtils (random-field
  noise only), QuadParticleRenderState (package move + buffer-plumbing
  signature shuffle; `add` identical), ItemStackRenderState
  (`pickParticleMaterial` → `pickParticleIcon`, same random pick; comment
  updated — its visitExtents localTransform step and the empty-model
  bounding-box difference have no Rust mirror, recorded in the triage),
  ParticleResources (only the two mob-growth registrations removed).
  New tests: the four lights snapshot to 240 under a full-sky probe; the
  Glow family reports the translucent layer.
  Referrals: item 354 (TerrainParticle ctor layer — the `Sprite::Named`
  submit arm still uses the sprite's own translucency, the 26.1 bySprite
  behaviour), item 192 (BreakingItemParticle shares the item-sprite side;
  the Glow `translucent_layer()` fix lands here).
  Rank 2 now 207/492 (43 changed, 164 no-change).
  work-items.json rank2.closed = 207 on 2026-11-20.

Item 29 (game/stations.rs, rank 2) — CHANGED. `StonecutterScreen.containerChanged`:
26.1 resets scrollOffs/startIndex on every container change; 1.21.11 resets only
when displayRecipes goes false (no input). The Rust reset when the input was gone
OR the recipes were empty — a hybrid neither version has — so swapping stone for
dirt reset the scroll where 1.21.11 keeps it. Fixed: `stonecutter_changed` now
resets only when `inputs[0]` is none. New test: scroll kept across an input swap
to a no-recipe item, reset when the input leaves. No-change verdicts, all
jar-verified: the enchanting-book texture path moved in 1.21.11
(`textures/entity/enchanting_table_book.png`, was `.../enchantment/...`) but the
Rust already uses the 1.21.11 path; the stonecutter scroller hover region shrank
to the 15-px thumb and lost the NOT_ALLOWED cursor, but stations.rs has no
cursor-shape code for it (the grab region is identical in both versions);
MapId.key() is `map_<id>` in 1.21.11 vs `maps/<id>` in 26.1 — no Rust mirror in
stations.rs, and the tree already uses the 1.21.11 form (note: the javap
normalizer hides this — the two MapId dumps are byte-identical; the decompile
is the reliable source for string-constant changes); ClientBundleTooltip's
weight-error gate and weight==0 empty text are 26.x-only — the Rust already
matches 1.21.11 (no gate; empty label only when the contents are empty);
ChargedProjectiles/ItemContainerContents are ItemStackTemplate→ItemStack API
reshapes with identical tooltip logic. Gap recorded: InstrumentComponent is an
EitherHolder in 1.21.11 and adds its tooltip line only when it unwraps, but the
Rust always renders the line — the client carries no instrument registry, and
the protocol adapter drops key-named instruments at parse, so an unresolvable
instrument never reaches the tooltip through the wire (except that a goat horn
then falls back to the ponder default where 1.21.11 shows no line — adapter
layer, out of scope). Everything else is render-plumbing renames
(extractBackground→renderBg, GuiGraphicsExtractor→GuiGraphics, ...) with
identical bodies. Method-name diff run explicitly: no missing method with
behavioural content.
Rank 2 now 208/492 (44 changed, 164 no-change).
work-items.json rank2.closed = 208 on 2026-11-20.

## Item 30 — crafting.rs (rank 2, CHANGED)

26.x made the special recipes data-driven (ingredients + result template
from JSON); 1.21.11 hardcodes them (category-only JSONs). The Rust mirrored
the 26.x forms, so on 1.21.11's data most specials did not load at all:
every loader arm requiring JSON fields returned None on a category-only
file, and `crafting_special_armordye` / `crafting_special_tippedarrow` /
`crafting_special_mapcloning` were unknown types. Verified against the
1.21.11 server jar's data (1470 recipe files): all 1.21.11 special JSONs
are category-only; the 32 transmute recipes (bundle + shulker dyeing) do
load. The port is mostly a loader rework: the loader now hardcodes the
1.21.11 ingredients/results per type (vanilla's `CustomRecipe.Serializer`
codec reads only `category` and ignores other fields, so ignoring the JSON
fields is exactly vanilla's behaviour). `crafting_dye` / `crafting_imbue`
arms removed; `crafting_special_armordye` / `crafting_special_tippedarrow`
/ `crafting_special_mapcloning` added. Transmute bounds hardcoded (1, 1)
and add_count false — `material_count` / `add_material_count_to_result`
are 26.x-only fields. Real behaviour fixes: the Transmute match loop now
follows the 1.21.11 form (a stack matching both input and material serves
as the material once the input is taken; 26.x fails it); BookCloning's
source test is the written-book component only, not an item (the `source`
field is gone; `tryCraftCopy`'s generation >= 2 gate is mirrored in
assemble); BannerDuplicate's result is `Option<String>` (None keeps the
source banner's item — 1.21.11 ships one recipe, 26.x one per colour);
new `MapCloning` variant (one MAP_ID stack + blank maps → copies, original
consumed — 26.x does this as a transmute). Dye/Imbue lost their
book_displays and placement arms: `ArmorDyeRecipe` / `TippedArrowRecipe`
are CustomRecipes with no display (`Recipe.display` defaults to empty)
and NOT_PLACEABLE in 1.21.11; the DyedSlotDemo / WithAnyPotion displays
are 26.x-only. Loader details jar-verified: 1.21.11 has NO
`#minecraft:dyes` tag (the dye ingredient is the explicit 16-item list);
`#book_cloning_target` is writable_book only; the firework star's creeper
shape is `#minecraft:skulls` — SHAPE_BY_ITEM has seven heads INCLUDING
dragon head (an earlier note said six; the jar says seven). No-change
verdicts, all jar-verified: RepairItem (curse-merge differs in form only,
same outcome); ShapedRecipe / ShapedRecipePattern / Ingredient /
StackedContents (logic identical); CustomRecipe showNotification (26.1
overrides it false, 1.21.11 takes the default true — the Rust already
reads it from the JSON with default true); registration order (the Rust's
path-then-namespace sort matches 1.21.11's `RecipeManager.prepare`
TreeMap<Identifier> exactly; only the test comment cited the 26.x-only
class, re-pointed). Pre-existing gaps in BOTH versions, recorded not
fixed: MapExtending's exploration-map rejection (no exploration flag is
plumbed through `find_with`'s callers); DecoratedPot's all-brick case
(both versions always set POT_DECORATIONS, the Rust omits it).
Referrals: item 752 owns ArmorDyeRecipe / TippedArrowRecipe /
TransmuteResult / MapCloningRecipe — their behaviour is implemented here,
its triage should find them done; the SlotDisplay wire codec
(`horizon-protocol/src/packets/recipes.rs`) still carries the 26.x-only
WithAnyPotion / OnlyWithComponent variants — that file is another item's,
and the codec rewire is the known adapter-tables follow-up. Method-name
diff run explicitly: no missing method with behavioural content.
Rank 2 now 209/492 (45 changed, 164 no-change).
work-items.json rank2.closed = 209 on 2026-11-20.

## Item 31 — dedicated.rs (rank 2, CHANGED)

MinecraftServer / DedicatedServer / WorldLoader, 5 classes, 69 methods,
13 named. Same theme as item 24's save.rs: the 26.x split reversal (game
rules, world-gen settings, random sequences, weather, boss events and
scheduled events live in server-wide split storage in 26.1, in level.dat /
the overworld level in 1.21.11) — most of that fallout is in the load/save
path, other items' files. The one change in this file: 26.1's
DedicatedServer.initServer ends with `saveEverything(false, true, true)`;
1.21.11 does not (verified in the decompile diff and javap — the
saveEverything call is a 26.1-only line). The Rust mirrored the 26.1 call
at the end of run_dedicated_inner; deleted it (save + its three "Saving
chunks for level" log lines). A 1.21.11 server's first write is now the
first autosave or the stop, exactly as vanilla 1.21.11 — loadLevel writes
nothing in either version. `log_saved_levels` stays: both stop paths still
use it, where vanilla saves in both versions. Not unit-testable
(run_dedicated_inner needs a full server); verified by cargo check, runs
on CI.
No-change verdicts, all jar-verified: setInitialSpawn (settle_spawn),
stopServer, tickServer, tickConnection, prepareLevels, the runServer loop,
kickUnlistedPlayers, createCommandSourceStack, buildServerStatus /
buildPlayerStatus, convertOldUsers, isUnderSpawnProtection, all property
setters — identical logic in both versions (renames + protected → public
only). WorldLoader's async → sync registry load is an implementation
detail, same steps in the same order. The dedicated.rs tick-loop low-disk
warning is already the 1.21.11 adaptation (its comment says 1.21.11 has no
packet to warn the operators with); the dedicated/*.rs submodules hold no
26.x-only mirrors. The saveAllChunks log lines already use the raw
dimension names (1.21.11's form, not 26.1's DIM-1 mapping).
Referrals (real vanilla differences, other files' scope): the per-level
time sync — 26.1 does one global sync every 20 ticks with an empty clocks
map (`forceGameTimeSynchronization`), 1.21.11 syncs per level with game
time + day time + ADVANCE_TIME (`synchronizeTime`); the Rust mirrors 26.1
in lan/hub.rs `sync_world` / `time_packet`, and through the adapter a
vanilla 1.21.11 client would see its day time reset to 0 and frozen every
20 ticks (the wire shape itself is already translated both ways by the
adapter — the remaining work is behavioural, hub.rs items 45/213/214/226/
241/273/336/337/374/395/424/485/498/794); the 1.21.11-only legacy world
border migration in createLevels (no Rust mirror anywhere — world_session
items 262/374); chunk_io.rs still calls warn_on_low_disk_space from the
26.1-only reportChunkLoad/SaveFailure callers (chunk_io items 197/214/241/
262/651); onGameRuleChanged's ADVANCE_TIME branch is 26.x-only and has no
Rust mirror (commands.rs set_game_rule sends no time packet — correct for
1.21.11). Gaps recorded with no owning item yet: setupDebugLevel (no
debug-world path in the Rust), saveDebugReport's path form,
fillServerSystemReport's "Dedicated Server (map_server.txt)" string,
OldUsersConverter's extra player-file check in 1.21.11's initServer
(`serverReadyAfterUserconversion` = lists removed AND old players
converted, verified from the 1.21.11 server jar), the 1.21.11-only
fatalException machinery (the Rust uses catch_unwind +
stop_after_crash). Method-name diff run explicitly: 26.1-only
clockManager, forceGameTimeSynchronization, getDataStorage, getGameRules,
getGlobalGameRules, getRandomSequence(s), getScheduledEvents,
getWeatherData, getWorldGenSettings, sendLowDiskSpaceWarning,
setWeatherParameters, warnOnLowDiskSpace; 1.21.11-only
forceTimeSynchronization, managedBlock, setFatalException,
synchronizeTime, throwIfFatalException — all accounted for.
Rank 2 now 210/492 (46 changed, 164 no-change).
work-items.json rank2.closed = 210 on 2026-11-20.

## Item 32 — menu.rs (rank 2, NO-CHANGE)

AbstractContainerScreen / StonecutterMenu / LecternBlockEntity /
SmithingMenu / BrewingStandMenu$PotionSlot / CraftingRecipe (+
CraftingBookInfo), 7 classes, 50 methods, 13 named. Harness menu_click.
No behaviour change: every menu-side rule is identical in both versions.
The differences are render-plumbing renames (extract* → render*,
GuiGraphicsExtractor → GuiGraphics, identical bodies), two method
inlinings with byte-identical bodies (shouldAddSlotToQuickCraft into
mouseDragged, quickCraftToSlots into mouseReleased — verified against the
26.1 helpers), a pure rename (ContainerInput → ClickType, same seven
values), a signature change with identical math
(getQuickCraftPlaceCount takes the set in 1.21.11, the count in 26.1 —
decompiled AbstractContainerMenu in both versions: type 0 →
floor(count/size), 1 → 1, 2 → maxStackSize, default → count), and
API-shape changes with identical outcomes (smithing/stonecutter assemble
takes registryAccess in 1.21.11 — the Rust has no assemble call there,
it clones precomputed results; defaultCraftingReminder returns the stack
directly in 1.21.11 vs a template in 26.1 — the Rust maps item → 1-count
stack either way; LecternBlockEntity.resolveBook wraps the source in a
ResolutionContext in 26.1 — no Rust mirror). The Rust mirrors verified:
`quick_craft_accepts` (the shouldAddSlotToQuickCraft gate, exact),
`quick_craft_preview` (place-count math + remainder, exact), `click`'s
QUICK_CRAFT state machine (status 0/1/2, single-slot fallback to Pickup),
stonecutter `button` (same index → false, invalid index still true),
`stone_changed` / `stone_setup_result`, `smith_result` / `smith_error`
(all inputs filled, no result), lectern `set_data` (clamp 0..pageCount-1)
and `lectern_button` (100+n / 1 / 2 / 3 take-book), brewing `Role::Potion`
(is_bottle = recipe inputs ∪ glass bottle, exactly vanilla's four items;
the `#minecraft:brewing_potion_inputs` tag clause is dead — the tag does
not exist in 1.21.11's data, checked in server-game.jar). Screen-framework
differences with no Rust mirror, recorded: mouseReleased returns literal
`true` in 1.21.11 vs `super.mouseReleased(event)` in 26.1 (the Rust's
inventory_mouse_release returns ()); the tooltip step moves from the
base render() (26.1) to the subclass render() (1.21.11 — renderTooltip
is defined but never called in the 1.21.11 base class); renderBg /
renderBackground are 1.21.11-only; DEFAULT_IMAGE_WIDTH/HEIGHT become
mutable fields with the same 176/166 defaults. The one fix is cosmetic:
`quick_craft_preview`'s doc comment cited the 26.x name `extractSlot`,
now `renderSlot` (the only stale 26.x citation in menu.rs).
Referrals: inventory_input.rs items (191/226/315) for the mouseReleased
return; container_screen.rs items (226/262/315/342) for the tooltip
caller; books.rs items for the unmirrored lectern resolveBook.
Method-name diff run explicitly: only the extract*/render* renames, the
two inlined helpers, renderBackground/renderBg, and the 26.x-only
CraftingBookInfo record. Javap string-constant scan: no string changes.
Rank 2 now 211/492 (46 changed, 165 no-change).
work-items.json rank2.closed = 211 on 2026-11-20.

## Item 33 — mobs/fox.rs (rank 2, CHANGED)

Fox + 20 nested goal/selector classes + CaveVines, 22 classes, 45
methods, 13 named. Harnesses entity_model, mob_ai, mob_attributes,
natural_spawn, spawner, trading, trial_spawner, vault. Three real
differences, all jar-verified:

1. Baby foxes are smaller: `Fox.BABY_DIMENSIONS` scales 0.5 in 1.21.11
   (0.6 in 26.1; the 26.1 `BABY_SCALE` constant is gone), eyes at
   0.2975 in both (adult eye height 0.4F, verified from both jars'
   `EntityType.FOX` builders). The Fox def in `mobs/species.rs` was
   `Baby::Scale(0.6, Some(0.34375))` — the eye height was a pre-existing
   error in both versions' mirror (0.34375 is the wolf's baby eye
   height), corrected in the same literal to
   `Baby::Scale(0.5, Some(0.2975))`.
2. The crouch caps at 3.0, not 5.0: 1.21.11 has no `MAX_CROUCH_AMOUNT`
   constant — `Fox.tick` and `isFullyCrouched` use a literal 3.0F, so a
   fox is fully crouched after 15 ticks (26.1: 25) and
   `FoxPounceGoal` can start 10 ticks sooner. `MAX_CROUCH_AMOUNT`
   5.0 → 3.0 (the named constant stays so both call sites stay in
   step) + doc fixes.
3. The pounce pitch drops the 6.5x upwards bias: 26.1's
   `FoxPounceGoal.tick` multiplies a rising fox's y-velocity by 6.5
   and skips a near-zero length; 1.21.11 tilts to the plain velocity
   pitch `signum(-y) * acos(horizontal / length)`. Ported as a pure
   `pounce_pitch` helper (the goal tick needs a full `Cx`); the call
   site's % 360 + clamp(-90, 90) normalisation is version-neutral and
   stays. The zero-velocity case would set xRot to NaN in vanilla
   1.21.11 (no guard) — unreachable in practice (the goal exits a
   still fox on the ground with pitch 0), and the Rust's clamp passes
   NaN through unchanged.

No-change verdicts (all jar-verified): the flags byte, aiStep eating
(>600 ticks, FOOD + CONSUMABLE), equipment odds, handleEntityEvent 45
(8 item particles; ItemStackTemplate → ItemStack is API-only), breed /
spawn / finalizeSpawn / attributes / save-load / trust slots,
canHoldItem / pickUpItem / spitOutItem (delay 40), tick wake/unsit /
faceplant particles, isFood, head roll (0.11π), clearStates, canMove,
the ambient screech, dropAllDeathLoot, isPathClear, FoxLookControl
(matches `resets_x_rot` exactly), Variant (RED 0 / SNOW 1,
`#spawns_snow_foxes`), and every goal — only the pounce pitch branch
differs. `CaveVines.use` / `hasGlowBerries` / `emission` are identical;
the harvest loot table read from both server jars is byte-identical
(1 roll, exactly 1 glow berry), so the Rust's fixed 1-berry drop is
exact. The `PathType` renames (DAMAGING_IN_NEIGHBOR → DANGER_OTHER,
DAMAGING → DAMAGE_OTHER) are pure renames; the Rust's `PathType`
already uses the 1.21.11 names. Method-name diff run explicitly: only
synthetic `access$400` (private → package-private visibility). Javap
string-constant scan: no string changes; the float constants confirm
exactly the three differences above.
Pre-existing gaps (both versions, recorded not fixed): `random_teleport`
uses `full_collision` where `LivingEntity.randomTeleport` uses
`blocksMotion()` (verified identical in both jars; fixing needs a new
blocks-motion fact in the prepared-data pipeline — no such fact exists
in `StateFacts`); the fox's ctor path malus (DANGER_OTHER / DAMAGE_OTHER
0.0) has no Rust mirror, so foxes treat damage-other blocks as blocked
(default -1.0) instead of free — two `set_malus` lines in `register()`
for a pathing item; `clear_states` clears the whole flags byte
including POUNCING / DEFENDING where vanilla clears five flags only;
DefendTrustedTargetGoal and SeekShelter's village check remain
unported (already noted in-code).
Tests: `a_crouching_fox_is_fully_crouched_at_three`,
`a_pouncing_fox_tilts_with_its_velocity` (45° rise → -45°, where 26.x
gave ≈ -81°), `a_baby_fox_is_half_its_adult_box`. All 7 fox tests pass
locally (`cargo test -p horizon-client-lib --lib --no-default-features
mobs::fox::`); `cargo check -p horizon-client-lib --lib --tests
--no-default-features` 0 warnings.
Rank 2 now 212/492 (47 changed, 165 no-change).
work-items.json rank2.closed = 212 on 2026-11-20.
