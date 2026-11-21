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

## Item 34 — mobs.rs (rank 2, CHANGED)

RenderTypes, EntityModel, AvatarRenderState, Cow, Wolf, Piglin, RingBell,
ShootTongue, Evoker$EvokerWololoSpellGoal, StartHuntingHoglin,
FollowPlayerRiddenEntityGoal (+ nested FollowEntityGoal): 12 classes, 72
methods, 10 named. Harnesses entity_model, mob_ai, mob_attributes,
natural_spawn, spawner, trading, trial_spawner, vault. Two real
differences, both jar-verified:

1. Baby piglins are half their adult box: 26.1's `BABY_DIMENSIONS` is an
   absolute 0.49 x 0.99, eyes 0.78 (with a vehicle attachment); 1.21.11
   scales the type's dimensions 0.5 with eyes at 0.97 (adult piglin 0.6 x
   1.95, eye 1.79, verified identical in both jars' `EntityType.PIGLIN`
   builders), so the baby box is 0.3 x 0.975. The Piglin def in
   `mobs/species.rs` was `Baby::Abs(0.49, 0.98, 0.78)` (the 0.98 a
   pre-existing 0.01 slip against vanilla 26.1's 0.99) — now
   `Baby::Scale(0.5, Some(0.97))`, matching the fox (item 33) and villager
   (item 20) precedents for the same vanilla `scale(0.5F)` shape.
2. `StartHuntingHoglin` gates on the bare presence of the
   `NEAREST_VISIBLE_ADULT_PIGLINS` memory: 26.1 filters the list for a
   piglin that hunted recently (`Optional.filter`, bytecode-verified);
   1.21.11 uses `.map(...).isPresent()` (bytecode-verified), which is true
   whenever the memory is present whatever the mapped boolean is — the
   `hasHuntedRecently` check is dead. `Brain.setMemoryInternal` erases a
   memory whose value is an empty collection (verified in the 1.21.11 jar)
   and the sensor sets the memory to its collected list every tick, so the
   memory is present iff >=1 adult piglin is visible: a piglin starts a
   hoglin hunt only when NO adult piglin is visible. A genuine Mojang
   filter→map regression; the jar is the oracle, so it is ported as-is:
   `HuntHoglin.can_use` in `mobs/piglin.rs` drops `&& s.hunted_recently`
   from the nearby-adult-piglin predicate, with the doc comment rewritten.

No-change verdicts (all jar-verified): the wolf texture, sounds and step
sound, cow sound variants, wolf collar dye, wolf feeding, the
FollowPlayerRiddenEntityGoal removal, the EntityModel default render type,
AvatarRenderState.scoreText, Piglin.getSlot and the piglin brain memory
inventory were all already 1.21.11 in the Rust (see the triage note for the
per-mirror evidence). Wolf feeding is a NO-DIFF after all: 26.1's
`TamableAnimal.feed(.., 2.0F, 2.0F)` heals 2.0 x nutrition (or 2.0) and
calls `playEatingSound()`, which is an EMPTY method in `Animal` (verified
in the 26.1 jar); 1.21.11 inlines the same heal with no sound. The
#wolf_collar_dyes tag exists in 26.1 (`#minecraft:dyes`) and is absent in
1.21.11; on vanilla data both accept exactly the 16 dyes. RingBell (0.95F,
within 3.0); ShootTongue (1.75F, eat at >=6 ticks, unreachable list max 5,
100L expiry); Evoker wololo (fangs 5+8 close / 16 line far, 3 vexes,
blue→red, MOB_GRIEFING gate); Wolf attributes/save/finalizeSpawn/breed/
canMate/wantsToAttack/tail angle/anger/spawn rules; Wolf `actuallyHurt`
armor-crack particles (Item vs ItemStack API only); Piglin save/inventory/
attributes/spawn/finalizeSpawn/mobInteract/`holdInOffHand`/`startRiding`
(baby hoglin top-passenger, 3 max); `Entity.is(Type)` vs `getType() ==
Type` equivalent (Wolf PREY_SELECTOR, Piglin startRiding). The `PathType`
renames are pure renames. Method-name diff run explicitly: the 26.1-only /
1.21.11-only methods are exactly the RenderTypes reshuffle, the cow/wolf
sound-variant accessors, `Piglin.getSlot`/`makeBrain` vs `brainProvider`,
and the removed goal class. Javap string-constant scan: only RenderType
name strings; no gameplay string changes.
Pre-existing gaps (both versions, recorded not fixed): the Rust's
`HuntHoglin.start()` gives nearby adult piglins the `HUNTED_RECENTLY` wait
as well as the target, where vanilla's `broadcastAngerTarget` sets only the
anger target (item 19's scope); the Rust's `nearby` gate does not check
line of sight, where the vanilla memory holds only VISIBLE piglins
(version-neutral approximation).
Referrals: item 48 (EnderDragonRenderer — 26.1's `entityCutoutDissolve`
is gone; `dragon.rs:3448` still cites it), item 56 (BannerRenderer —
`BANNER_PATTERN` gone), item 343 (EndCrystalRenderer — `END_CRYSTAL_BEAM`
gone); the item-render sheets and the 1.21.11-only weather render types
belong to their own items (the Rust has no `RenderTypes` table to rewire).
Tests: `a_baby_piglin_is_half_its_adult_box` (real assertion, passes
locally), `a_piglin_does_not_hunt_while_another_adult_piglin_is_near`
(needs the world fixture — early-returns locally, runs on CI; `sees` was
verified to resolve through `others` without `who_lookup`, and missing
chunks read as air for line of sight). All 15 piglin tests pass locally
(`cargo test -p horizon-client-lib --lib --no-default-features
mobs::piglin::`); `cargo check -p horizon-client-lib --lib --tests
--no-default-features` 0 warnings; both touched files `rustfmt --check`
clean.
Rank 2 now 213/492 (48 changed, 165 no-change).
work-items.json rank2.closed = 213 on 2026-11-20.

## Item 35 — game/commands/items.rs (rank 2, CHANGED)

ItemStack, ItemInput, SlotCollection$Filtered, ItemCommands,
CompositeSlotSource, TransformedSlotSource, ListOperation$Type: 7 classes,
61 methods, 11 named. Harnesses brewing, creative_mining,
enchant_durability, enchanting, furnace, recipe, stack_mining,
trade_components. One real difference, jar-verified:

`ItemInput.createItemStack` drops strict validation in 1.21.11: 26.1
builds the stack, throws `arguments.item.overstacked` when the count
exceeds the max stack size, then calls `ItemStack.validateStrict` and
throws `arguments.item.malformed` on failure; 1.21.11 takes a
`checkOverstacked` boolean and never validates. The /item command passes
false for the no-count form and true for the count form, so the overstack
behaviour is unchanged (count 1 can never exceed a max_stack_size that
parses in [1;99]). The observable difference is the malformed rejection:
`/item replace ... with iron_helmet[max_stack_size=2]` (damageable +
stackable) is rejected in 26.1 and accepted in 1.21.11. The Rust's
`item_from_input` mirrored the 26.1 form — the malformed check and its
`item_validation_components` helper (a prototype+patch view used only by
that check) are removed; the overstack check stays. `/give` never had the
check — already 1.21.11.

No-change verdicts (all jar-verified): every `ItemCommands` body
(modify/set/transfer/applyModifier/getItemInSlot/getContainer);
`SlotCollection`/`CompositeSlotSource`/`TransformedSlotSource` (the
`validate` refactor only changes validation error paths, which the Rust
does not mirror; `Filtered.filter`'s `Predicate.and` is behaviourally
identical to 26.1's manual `&&`); `ListOperation` fully identical
(Append/Insert/ReplaceAll/ReplaceSection/StandAlone, overflow + bounds
errors); `ItemStack` behaviour throughout — use/finishUsingItem, durability,
copy/transmute/matches/hash, tooltips (component order and all branches
identical), foil/rarity/enchantments, getDisplayName (26.1's
`ItemStackTemplate` vs 1.21.11's plain `ShowItem` — same visible hover),
count/limitSize/consume, `applyComponentsAndValidate` (validates +
restores on failure in both). `ItemInput.serialize` (1.21.11-only) has no
callers in either 1.21.11 jar (constant-pool scan) — dead API surface.
The `validateComponents` narrowing (26.1 validates bundle + charged-
projectile contents; 1.21.11 only the container) has no gameplay mirror in
the Rust. Method-name diff run explicitly: the 26.1-only / 1.21.11-only
methods are the ItemInput record→class change, the ItemInstance interface
(26.1-only; `count()`, the `is` overloads, `getMaxStackSize`, `getTags`
live there in 26.1, directly on ItemStack in 1.21.11), the
`typeHolder`→`getItemHolder` rename, the entity-representation API (below),
and `validateComponents`' visibility. Javap string-constant scan: only
`arguments.item.malformed` (26.1-only) is a real string change.
Pre-existing gaps (both versions, recorded not fixed): the Rust never reads
a stack's `minecraft:damage_type` component (no `getDamageSource` mirror),
so the 1.21.11 EitherHolder-unwrap form vs 26.1's plain Holder has nothing
to rewire (same EitherHolder-unwrap-gate shape as the InstrumentComponent
gap from item 29); the 1.21.11-only entity-representation API (`isFramed`/
`getFrame`/`setEntityRepresentation`) has no Rust mirror — its only
consumer is `MapItemSavedData` (map rendering), a frames/map-item referral.
`MAP_CODEC` (`CODEC_WITH_BOUND_COMPONENTS` vs plain `CODEC`) and the
1.21.11-only `SINGLE_ITEM_CODEC`/`STRICT_CODEC` constants are codec-layer
surface for the item-codec items.
Tests: `damageable_and_stackable_compositions_are_accepted_but_
overstacking_rejects_before_mutation` (rewritten from the old reject test;
`#[ignore]`, needs prepared resources — runs on CI) now accepts the three
damageable+stackable compositions and still rejects an overstacked count
before the slot is touched; the `item_validation_components` test went
away with the helper. `cargo check -p horizon-client --bin horizon-client
--tests --no-default-features` 0 warnings; items.rs `rustfmt --check` clean.
Rank 2 now 214/492 (49 changed, 165 no-change).
work-items.json rank2.closed = 214 on 2026-11-20.

## Item 36 — game/game_tests.rs (rank 2, CHANGED)

GameTestHelper, GameTestServer, GameTestRunner (+$1), GameTestInfo,
GameTestSequence (+$Condition), StructureGridSpawner, TestCommand,
TestEnvironmentDefinition (+AllOf/SetGameRules/Weather/ClockTime/Timelines/
TestBlockPlaceContext), BlockBasedTestInstance, GameTestInstance: 25
classes, 106 methods, 6 named. Harnesses mob_ai, redstone_catalog, sculk,
trading. Three real differences, all jar-verified:

Test environments are the 1.21.11 model, not the 26.1 one. 26.1's
`TestEnvironmentDefinition<SavedDataType>` is generic — `setup` returns
saved data, `teardown(level, saved)` restores it, the runner keeps an
`Activation<T>` handle. 1.21.11's is plain `void setup` / `void teardown`,
no `Activation`, and the runner keeps the definition itself. The type
registry differs: 26.1 has `clock_time` (WorldClock, saved/restored) and
`timeline_attributes`; 1.21.11 has `time_of_day` (int `time`, `setDayTime`,
NO teardown — the time stays changed). Teardown semantics differ per
type: `AllOf` tears down reversed in 26.1, in order in 1.21.11;
`SetGameRules` restores the originals in 26.1 but resets each touched
rule to its DEFAULT in 1.21.11 (`resetRule`); `Weather` re-applies the
previous type after `resetWeatherCycle` in 26.1 but only resets the
cycle in 1.21.11 (`resetWeatherCycle` verified from the 1.21.11 jar
bytecode: rainTime 0, raining false, thunderTime 0, thundering false;
clear_time untouched). The Rust's `Saved` + `Activation` are deleted;
`Environment` drops `ClockTime`/`Timelines` and gains `TimeOfDay`;
`GameTests.environment` holds the definition; `set_up_test_environment` /
`tear_down_test_environment` take no saved data. `difficulty` is a
Horizon extra — no vanilla type in either version's bootstrap — kept,
with teardown resetting to normal (2) on the 1.21.11 reset-to-default
pattern. The default environment is the empty `all_of` in both jars.

`GameTestInfo.tick`: the chunks check gates everything in 1.21.11.
26.1 runs `tickInternal()` + listeners whenever the test is done, even
with an error set before the chunks loaded (the chunks check is inside
the `else`); 1.21.11 wraps both `tickInternal()` and the listener
notification in the chunks-loaded check, so a test failing before its
chunks load is done but never notifies (and `isDone()` returns early on
later ticks, so it never notifies at all — the batch stalls). The Rust's
`tick_test` drops the `else`: the chunks check now applies with an
error set too, and the stall consequence is mirrored (`test_finished`
never runs, `batch_pending` never reaches 0).

Barriers are removed on failure too in 1.21.11. 26.1 removes them only
in `testPassed`; 1.21.11 removes them in `testCompleted`, which runs
from `testPassed` AND `testFailed` (non-halt), and the halt branch
removes them too — always on completion. The Rust's `test_finished`
now removes barriers unconditionally (halt-path order differs from
vanilla's — barriers first here, last there — but the operations touch
independent state, so it is unobservable).

Also fixed: the `getTest*` accessors are gone in 1.21.11
(`TestInstanceBlockEntity` has only `getStructureBoundingBox()` /
`getStructureBounds()` there; 26.1 has those AND the padded
`getTestBoundingBox()` / `getTestBounds()`; `GameTestInfo` has only
`getStructureBounds()` in both) — comments in game_tests.rs and
test_instances.rs updated; the Rust already used the unpadded box
everywhere (`TestDef` has no `padding` field — `GameTestInstance.padding()`
is 26.1-only). The game_tests.rs header's "Not ported" claims were stale
on two of three counts (environments ARE ported, succeed DOES discard
the finished test's entities); only the sequences claim stands.

No-change verdicts (all jar-verified): `GameTestHelper` has NO Rust
mirror (the Rust runs block-based tests and data-defined function tests,
not Java test functions), so its 26.1-only helpers (`despawnItem`,
`discard`, `placeBlock` x2 + `TestBlockPlaceContext`, the `moveTo`
overload, `assertBlockPresent(Block)`, `setTime`, `runBeforeTestEnd`,
`getBoundsWithPadding`, `getItemName`), the `setBiome` bounds difference
(padded in 26.1, plain in 1.21.11) and the item-name error-string
difference (ITEM_NAME component vs `item.getName()`) need no change;
`GameTestSequence` is not ported (`thenWaitAtLeast` 26.1-only);
`GameTestServer` has no Rust mirror (no gametest-server binary;
`repeatCount`/`multiplyTest` 26.1-only); `StructureGridSpawner`,
`MultipleTestTracker`, `TestCommand` identical except the accessor
rename; the runner's builder/batcher/spawners identical; `GameTestInfo`
otherwise identical (startExecution tick math, placeStructure,
tickInternal, succeed's entity discard); `BlockBasedTestInstance` /
`GameTestInstance` identical except generics. `Weather.Type` parameters
identical. Method-name diff run explicitly from the javap output: the
only 1.21.11-only method in the 25 classes is `SetGameRules.resetRule`;
no 1.21.11-only classes (the report's classes are 26.1-keyed;
`TimeOfDay` came from the decompile).
Pre-existing, out of scope: the Rust recomputes `test_chunks_loaded`
every tick where vanilla latches `chunksLoaded` (both versions);
with no structure box the Rust treats chunks as loaded where vanilla
1.21.11 would NPE (graceful handling stands — mirroring a crash is
wrong); the 26.x WorldClock system (clock commands, chat markers) is
other items' scope — only the test-environment `clock_time` TYPE goes
away here.
Tests: `definitions_parse_with_references_and_nesting` and
`unknown_or_invalid_definitions_do_not_parse` updated (`time_of_day`
replaces `clock_time`; `clock_time`/`timeline_attributes` rejected,
`time > i32::MAX` rejected); new `#[ignore]`d
`teardown_resets_rules_and_weather_and_leaves_the_time` pins the 1.21.11
teardown semantics (rules reset to default, weather only resets the
cycle, time stays changed, difficulty resets to normal — needs prepared
resources in `HORIZON_TEST_RESOURCES`, runs with `-- --ignored` where
they exist). All 3 runnable environment tests pass, all 10 game_tests
unit tests pass. Verification note for later items: the game tests live
in the LIB test target (`cargo test -p horizon-client --lib
--no-default-features`), NOT the bin target — `cargo test -p
horizon-client --bin horizon-client` runs 0 tests, and `cargo check
--bin --tests` never compiles the `#[cfg(test)]` modules, so it cannot
validate test code. `cargo check -p horizon-client --lib --tests
--no-default-features` 0 warnings (the --no-default-features lib check
fits; only the full-feature one OOMs).
Rank 2 now 215/492 (50 changed, 165 no-change).
work-items.json rank2.closed = 215 on 2026-11-20.

## Item 37 — horizon-assets/item_bake.rs (rank 2, CHANGED)

ItemTransforms (+Deserializer, $1), ItemModelGenerator, ItemTransform
(+Deserializer), ItemStackRenderState$LayerRenderState, QuadCollection
(+$Builder), CuboidItemModelWrapper (+$Unbaked): 11 classes, 92 methods,
6 named. Harness first_person. One real difference, jar-verified — and
note the tooling caveat: the stock diffs mostly show "missing-side"
because 26.1 MOVED these classes (`ItemTransform`/`ItemTransforms`/
`ItemModelGenerator` live in `client/resources/model/cuboid/` in 26.1 vs
`client/renderer/block/model/` in 1.21.11; `QuadCollection` in
`resources/model/geometry/` vs `resources/model/`). The pairs were
decompiled manually with Vineflower and diffed across the move; the
wrapper pair is 26.1 `CuboidItemModelWrapper` vs 1.21.11
`BlockModelWrapper`.

Side-face UVs of an extruded sprite use swapped scales in 1.21.11.
The extrusion math is otherwise identical, but the side-face UV
construction differs, bytecode-verified on both jars (the UVs record is
(minU, minV, maxU, maxV) in both): 26.1 `bakeSideFaces` builds
`UVs(u0*xScale, v0*yScale, u1*xScale, v1*yScale)` (per-axis scaling);
1.21.11 `createSideElements` builds
`UVs(u0*xScale, v0*xScale, u1*yScale, v1*yScale)` (the middle two
scales swapped — minV by xScale, maxU by yScale). For square sprites
there is no difference; for non-square sprites the side-face UVs differ.
The Rust's `extrude_layer` used the 26.1 form — now the 1.21.11 form.
Doc citations in this file updated to the 1.21.11 names
(`BlockModelWrapper` for the wrapper, `processFrames` for the layer
extrusion, `transform.apply` in `submit`/`visitExtents` for the layer
matrix; the `item_render.rs` `computeExtents` citations belong to that
file's item or to 766, which covers the `BlockModelWrapper` rename).

No-change verdicts (all jar-verified): `ItemTransform.apply` identical
(left-hand mirrors translation.x, rotation.y/z; translate → rotate XYZ
→ scale → translate(-0.5)), Deserializer identical (translation x0.0625,
clamp ±5; scale clamp ±4); `ItemTransforms` identical (9-field record
incl. ON_SHELF, `getTransform` switch, Deserializer left-hand fallback
— mirrored in `model.rs`, where an explicit identity does NOT fall back,
matching vanilla's `==` on the NO_TRANSFORM instance);
`ItemDisplayContext` identical in both versions, ON_SHELF(9) included;
`QuadCollection` getAll/getQuads/`Builder.build` order identical
(unculled, then N/S/E/W/U/D) — the 26.1-only `materialFlags` family and
`Builder.addAll` have no Rust mirror; `ItemModelGenerator` otherwise
identical (LAYERS layer0..layer4, front/back faces, SOUTH_FACE_UVS
(0,0,16,16) / NORTH_FACE_UVS (16,0,0,16) bytecode-verified,
`getSideFaces`/`checkTransition`/`isTransparent`, `SideDirection`
mapping) — the 26.1-only `ItemLayerKey` cache and direct-FaceBakery
baking are plumbing (1.21.11 builds BlockElements and bakes via
`SimpleUnbakedGeometry`); `bake_item_model`/`bake_resolved` mirror
`bakeTopGeometry` + `ModelRenderProperties.fromResolvedModel`, the same
in both versions (`ModelRenderProperties` is in no report item); the
local transform is already handled — every Rust `Layer` is built with
`transform: Mat4::IDENTITY` (`item_model.rs`, "1.21.11 has no per-model
transformation"), so `layer_matrix` already behaves as 1.21.11;
`ItemStackRenderState`/`LayerRenderState` differences have no mirror in
item_bake.rs (`pickParticleMaterial` vs `pickParticleIcon`, `visitExtents`
via `applyTransform` vs direct `transform.apply` — no Rust visitExtents
mirror, already noted in item 28 — the empty-extents bounding-box guard,
tint storage `IntList` vs `int[]` + `prepareTintLayers`, `setRenderType`
+ `detectRenderType`, animated detection by material flag vs sprite
scan with the same outcome; foil handling and `computeExtents`
identical). Method-name diff run explicitly on the cross-package pairs:
only the expected plumbing renames (`bakeExtrudedSprite`/`bakeSideFaces`
vs `createSideElements`/`processFrames`, the material-flags family,
`addAll`, the layer-state setter renames).
Tests: new `non_square_sprite_extrudes_with_1_21_11_uv_scales` (2x1
sprite, one opaque pixel — the Up side face's vertex UVs must span
u {0.05, 0.9}, v {0.05, 0.9}; the 26.1 form would give u {0.05, 0.45},
v {0.1, 0.9}); all 3 item_bake tests and all 53 horizon-assets lib tests
pass. `rustfmt --edition 2024` on the leaf file.
Rank 2 now 216/492 (51 changed, 165 no-change).
work-items.json rank2.closed = 216 on 2026-11-20.

## Item 38 — mobs/animal_brain.rs (rank 2, CHANGED)

SnifferAi (+$Digging, $FeelingHappy, $FinishedDigging, $Scenting,
$Searching, $Sniffing, $1, $2, $3), AxolotlAi, HappyGhastAi,
CopperGolemAi, TadpoleAi, AnimalMakeLove, AnimalPanic,
FollowTemptation, PathfinderMob, ActivityData (removed), BabyFollowAdult,
BehaviorControl, SetEntityLookTarget, SetEntityLookTargetSometimes
(+$Ticker), AdultSensorAnyType: 25 classes, 88 methods, 6 named.
Harness mob_ai. No 26.1 → 1.21.11 behaviour change — the version delta
is API shape only — but two pre-existing Rust-vs-vanilla gaps in this
item's classes were fixed in passing, so the item counts as changed.

The brain-provider refactor is the main shape change: 26.1 Ai classes
return `List<ActivityData<T>>` via `getActivities()`; 1.21.11 uses
`Brain.makeBrain(Brain<T>)` + `addActivity` / `addActivityWithConditions`
/ `addActivityAndRemoveMemoriesWhenStopped` + `setCoreActivities` /
`setDefaultActivity` / `useDefaultActivity`, with explicit
`SENSOR_TYPES` / `MEMORY_TYPES` lists (in the Ai class for
CopperGolem/HappyGhast/Sniffer; in the entity class for Axolotl/Tadpole),
and `ActivityData` is removed entirely. `Brain.createPriorityPairs` is
the same sequential-numbering helper `ActivityData.create` used, so the
list form still numbers behaviours from the given priority. Activity
contents, priorities, speeds, conditions and erase-on-stop sets are
identical for all five Ai classes, verified field by field against the
Rust's five packages (sniffer core/dig/sniff/idle, copper golem core/idle
incl. the transport-gated RunOne, ghastling core/idle incl. the
fly-panic, tadpole core/idle, axolotl core/idle/fight/play-dead incl.
PLAY_DEAD stickiness and the 2400-tick hunting cooldown on FIGHT exit).

`TypedInstance` was removed in 1.21.11, so `entity.is(type)` /
`entity.is(tag)` became `getType() == type` / `getType().is(tag)` in
AnimalMakeLove, SetEntityLookTarget, SetEntityLookTargetSometimes and
AdultSensorAnyType — the same outcome in every case (26.1's
`TypedInstance.is(T)` is `typeHolder().value() == obj`, bytecode-verified;
the Rust's exact-type `species` comparison already matches 1.21.11).
`PathfinderMob.isPanicking` drops the `!brain.isBrainDead()` conjunct in
1.21.11; the Rust checks memory presence only, already 1.21.11.
`BehaviorControl.getRequiredMemories` is 26.1-only, no Rust mirror.
BabyFollowAdult differs only in UniformInt accessor names. AnimalPanic,
FollowTemptation and SnifferAi$1/$2/$3 have identical normalized javap
(the decompile diffs are parameter-name noise). The SnifferAi inner
classes differ only in visibility. Method-name diff run explicitly on
the javap dumps: the only differences are ActivityData's removal,
`getRequiredMemories`, and `getActivities` → `makeBrain` in the five Ai
classes.

Rust mirrors verified exact against 1.21.11 (decompile + bytecode): all
five activity packages; sensor orders for all five kinds (Tadpole:
[NEAREST_LIVING_ENTITIES, NEAREST_PLAYERS, HURT_BY, FROG_TEMPTATIONS]);
all MEMORY_TYPES slots; activity gating; FollowTemptation close
distances (sniffer baby 2.5 / adult 3.5, ghastling 3.0, default 2.5);
BabyFollowAdult (range [min, max+1), close = min-1, eye flags);
LookAtPlayerSometimes ticker (exact sample-1 semantics, 30-60
axolotl/tadpole, 40-80 copper golem); AnimalPanic fly vs land position
picks; RandomStroll variants (`stroll(0.15, false)` = `!isInWater`,
honoured for the axolotl; fly uses AirAndWaterRandomPos(10, 7, -2));
all six sniffer inner behaviours (start/tick/stop incl. the 9600 sniff
cooldown, the explored-position cap — vanilla limits to 20 THEN adds,
so 21 max, exactly like the Rust — and the sniffing-timeout walk target
1.25/0); MakeLove speeds (axolotl 0.2, else 1.0); axolotl speeds
(0.5/0.6 water, 0.15 land). The `#followable_friendly_mobs` tag JSON is
byte-identical in both jars (25 entries).

Fix 1 — `followable_friendly` omitted Armadillo and Bee (23 of the
tag's 25 entries), so a baby ghastling never followed an adult armadillo
or bee. Both added.

Fix 2 — `is_tempted` read the wrong memory: the Rust used
`tempting_player.present()` (the sensor memory: a player holding food is
near), but vanilla `Sniffer.isTempted()` is
`brain.getMemory(IS_TEMPTED).orElse(false)` (1.21.11 bytecode), and
IS_TEMPTED is set only by `FollowTemptation.start` and erased by its
stop (1.21.11 decompile; a jar-wide constant-pool scan shows
FollowTemptation is the only writer). The divergence window is the
100-tick temptation cooldown after FollowTemptation stops and while a
BREED_TARGET is present — vanilla's sniffer can scent/sniff/dig then,
the Rust's could not. Added `is_tempted: Slot<bool>` to `Memories`
(ticked, and reported in `describe` — the mob_ai oracle prints every
present memory via `brain.forEach`, so `is_tempted=true` now matches),
set in FollowTemptation start, cleared in its stop, read by
`is_tempted()`. Contained to animal_brain.rs (no callers outside the
file). Doc citations updated to the 1.21.11 names (`makeBrain` /
`addActivity`; `monster_brain.rs` and `warden_brain.rs` still cite
`getActivities` — their items' scope).

Referrals: the IS_TEMPTED conflation may exist in other brain files
(ArmadilloAi, CamelAi, Goat, Frog, NautilusAi reference IS_TEMPTED in
the jar) — check when those items come up; `TransportItemsBetweenContainers`
internals belong to that behaviour's own item (only its idle-priority-0
wiring was verified here); `PlayDead` / `ValidatePlayDead` are not in
this item's class list (only their activity wiring via AxolotlAi was
verified). The item-23 referral (HappyGhast/Hoglin brain-provider
refactor) is discharged for the HappyGhast half: contents identical,
Rust already matches 1.21.11.

Tests: new `followable_friendly_matches_the_tag` (all 25 tag species
true; Allay/Axolotl/Frog/Mooshroom/Turtle/Zombie/Creeper false) and
`tempted_memory_follows_follow_temptation_not_the_sensor`
(fixture-gated: FollowTemptation start sets IS_TEMPTED, stop clears it
while TEMPTING_PLAYER stays present with cooldown 100, Scenting blocked
while tempted and allowed after the stop). All 6 animal_brain tests pass
in the horizon-client-lib binary (the tempted-memory test early-returns
locally — no fixture — and runs for real on CI).
`cargo check -p horizon-client --lib --tests --no-default-features`
clean; `rustfmt --edition 2024 --check` clean (one long assert line
fixed by hand).
Rank 2 now 217/492 (52 changed, 165 no-change).
work-items.json rank2.closed = 217 on 2026-11-20.

## Item 39 — horizon-assets/item_model.rs (rank 2, CHANGED)

Transformation, BundleItem, RangeSelectItemModel$Unbaked,
ItemModelResolver, MissingItemModel, BuiltInRegistries, ItemModels,
RangeSelectItemModel, CompassAngleState$CompassTarget (+4 anon),
NeedleDirectionHelper, Time$TimeSource (+3 anon), LocalTime (+$Data),
RangeSelectItemModel$Entry: 21 classes, 67 methods, 8 named. Harness
item_model. Two real 26.1 → 1.21.11 behaviour differences, both fixed,
plus one stale citation.

`DATA_COMPONENT_TYPES` was the 26.3 list (116 entries), not 1.21.11's:
the 1.21.11 `DataComponents` registers exactly 104 types (verified
twice — the 1.21.11 class decompile and the datagen registries dump
agree exactly), and the oracle emits the sorted registry keys, so the
harness `components` row mismatched. Removed the 14 entries in neither
jar (attack_animation, block_transformer, brewing_fuel, compostable,
cooking_fuel, cushion/color, interact_animation, mob_visibility,
provides_pottery_pattern, sign_text_back, sign_text_front,
sulfur_cube_content, villager_food, waxed) and added the 2 missing
1.21.11 types (map_color, swing_animation — 26.x renamed
`swing_animation` to `attack_animation`, hence the swap). The list
stays sorted. (26.1 registers 110: the same 14 extras, missing 8 —
the 6 extra missing-vs-26.1 types, additional_trade_cost and the
cat/chicken/cow/pig sound variants plus dye, are gone by 1.21.11 too.)

`NeedleDirectionHelper.get` gains the entity-representation fallback in
1.21.11: when the owner is null, `owner =
itemStack.getEntityRepresentation()` (bytecode-verified; no cast needed
because `Entity` implements `ItemOwner` in 1.21.11), set only by
ItemEntity, ItemFrame and Display$ItemDisplay (jar-wide constant-pool
scan). Framed compasses/clocks therefore use the frame's position and
yaw. The Rust passed a dummy owner at [0,0,0] yaw 0 for framed items
(`game/frames.rs`), so framed needles pointed from the origin; it now
passes the frame's real position (block centre) and yaw
(`ItemFrame.setDirection`: horizontal → `get2DDataValue * 90` =
south 0 / west 90 / north 180 / east 270, bytecode-verified; vertical
→ yRot untouched, 0), reusing the pose block's `to_y` mapping, and the
entity type is now `f.kind()` (glow vs normal) instead of the hardcoded
"minecraft:item_frame". This changes only the needle (time is global;
the other owner-read properties keep their defaults), so no regression.
Dropped/thrown items already pass a real owner at the entity position
(creatures.rs) — needle already matched. Brush/campfire/shelf/spawner/
item-particle paths have no entity representation in vanilla either —
0.0 both sides.

`bundle_selected`'s doc citation updated: 1.21.11's
`BundleItem.getSelectedItem()` returns the int index (26.1's
`getSelectedItemIndex`); the stack getter is the new
`getSelectedItemStack()`, plus new `hasSelectedItem()`. The Rust
returns the stack and gates `BundleHasSelectedItem` on its
`.is_some()` — correct in all reachable states, since vanilla
`toggleSelectedItem` clamps out-of-bounds indices to -1 (verified from
the 1.21.11 `BundleContents$Mutable` decompile).

No-change verdicts (all jar-verified): Transformation's accessor renames
(getX, private IDENTITY behind `identity()`), null-tolerant matrix ctor
and the static `compose` removal — the Rust has no Transformation struct
(only `svd_decompose` in math.rs, mirroring `MatrixUtil`) and no
per-model transformation; RangeSelectItemModel's dropped `transformation`
field and param — the Rust decoder never parsed it (unknown fields
ignored, as DFU does) and every layer is identity, `update` /
`lastIndexLessOrEqual` identical; MissingItemModel's dropped transform
and `withTransform` — the Rust's `missing()` is already identity;
ItemModelResolver's `ModelManager` → two `Function` fields — all method
logic identical; ItemModels' `CuboidItemModelWrapper` →
`BlockModelWrapper` (the wrapper itself is item 766's scope);
CompassAngleState / Time / LocalTime logic identical (LocalTime.Data
visibility only); BundleItem's `weight()` DataResult → plain Fraction is
`BundleContents`' change (its own item) — the Rust's `bundle_fullness`
host impls sum count/max-stack-size, matching 1.21.11's no-error path;
`onDestroyed` `itemCopyStream()` → `itemsCopy()` rename;
`getAllBundleItemColors` / `getByColor` have no Rust caller;
BuiltInRegistries' loot/recipe type renames belong to the loot/recipe
items. Method-name diff run explicitly: only the expected differences
(the static `Transformation.compose` removal shares its name with the
instance method, so it does not show in the name diff — confirmed from
the decompile).

Referrals: item displays pass `None` as the owner (text_displays.rs) so
their needles are frozen at 0.0 — not fixed here, because passing the
display as the owner would regress MainHand/ContextEntityType/
CrossbowPull (currently correct with None); it needs the
entity-representation mechanism (a needle-only fallback in `Context`,
used by `time()`/`compass()` when owner is None). The same mechanism
would close the PRE-EXISTING owner-vs-null divergences for non-living
items (dropped items and frames pass `Some(owner)` where vanilla passes
null: MainHand returns "right" instead of falling back,
ContextEntityType returns the entity type instead of falling back,
CrossbowPull can be NaN for an uncharged crossbow instead of 0.0) —
pre-existing in both versions, not this item's version difference. The
item-35 entity-representation referral (frames/map-item) now has a
second consumer (NeedleDirectionHelper.get); the frames half is fixed
here via the owner path. `BundleContents` (weight API, `removeOne`,
`toggleSelectedItem`) is its own item (bundle.rs).

Tests: new `data_component_types_match_the_1_21_11_registry` (104
entries, sorted, contains map_color + swing_animation, excludes five
26.x-only names). All 19 item_model tests pass
(`cargo test -p horizon-assets --lib item_model::`).
`cargo check -p horizon-client --lib --tests --no-default-features`
clean; `rustfmt --edition 2024 --check` clean on the three changed
files (one diff reported only in item_model/foreign.rs — pre-existing
drift, rustfmt recursing through the module root, not touched).
Rank 2 now 218/492 (53 changed, 165 no-change).
work-items.json rank2.closed = 218 on 2026-11-20.

## Item 40 — horizon-protocol/hash_ops.rs (rank 2, NO-CHANGE)

HashOps, NbtOps, HashOps$MapHashBuilder, HashOps$ListHashBuilder:
4 classes, 87 methods, 6 named in Horizon. No 26.1 → 1.21.11 behaviour
change — the delta is visibility widening, one new default overload,
and compiler artifacts.

HashOps: fields `hashFunction` / `empty`, `hashMap(Stream)` and
`isEmpty`, and both builders go private → package-private. All bodies
identical (createNumeric switch, createByte(6)/Short(7)/Int(8)/Long(9)/
Float(10)/Double(11)/String(12)/Boolean, hashMap tag 2 + entries sorted
by key-then-value hash unsigned + tag 3, createList 4/5,
createByteList 14/15, createIntList 16/17, createLongList 18/19,
remove/set/update no-ops, mergeTo* gated on isEmpty, get*/convertTo
unsupported). The payload constants are outside every diff hunk —
unchanged — so the vanilla-printed hash values in the Rust tests
(printed from the 26.3 jars) still hold for 1.21.11. Builder ctors lose
the `Objects.requireNonNull` on the outer instance (javac artifact).

NbtOps: `convertTo` has the same 13 arms in the same order with the
same calls; only the switch compilation differs (26.1's guarded-pattern
re-dispatch loop vs 1.21.11's plain switch expression,
`typeSwitch:(LTag;I)` → `(LObject;I)` — javac version artifacts). All
other methods identical modulo parameter names (the 1.21.11 jar lost
debug info). Nested classes private → package-private. New in 1.21.11:
`ListCollector.acceptAll(Stream<Tag>)`, a default method delegating to
`acceptAll(Iterable)` — new API, no behaviour change, and the Rust has
no ListCollector mirror (it never merges NBT lists). Method-name diff
run explicitly on the raw javap dumps: no method added or removed in
any of the four classes (the `acceptAll(Stream)` addition is in the
nested `NbtOps$ListCollector`, visible in the decompile where
Vineflower inlines nested classes).

The Rust mirror matches 1.21.11 throughout: `empty`/`empty_map`/
`empty_list` hash the vanilla payloads; the create* methods match the
tag bytes and little-endian Guava puts (putFloat/putDouble write raw
bits; createString writes the UTF-16 length then unencoded chars);
`create_map` sorts by (key, value) as unsigned u32s (vanilla's
`padToLong` order); `from_nbt` matches `convertTo` arm for arm (the
`bool_keys` handling is a documented Horizon extension for the text
component codec); `RecordHash` mirrors MapHashBuilder. No mirror is
needed for `createNumeric` (callers dispatch by type), the no-op
`remove`/`set`/`update`, the `mergeTo*` family (the Rust hashes from
scratch), or the unsupported `get*`/`convertTo`.

Tests: all 6 hash_ops tests pass
(`cargo test -p horizon-protocol --lib hash_ops::`), including the
vanilla-printed hash values and the CRC-32C check value.
Rank 2 now 219/492 (53 changed, 166 no-change).
work-items.json rank2.closed = 219 on 2026-11-20.

## Item 41 — mobs/frog.rs (rank 2, CHANGED)

FrogAi, GoatAi, Frog, LongJumpToRandomPos, FrogAttackablesSensor, Croak,
LongJumpMidJump, LongJumpToPreferredBlock, TryLaySpawnOnFluidNearLand
(removed), Frog$FrogLookControl: 10 classes, 56 methods, 9 named.
Harnesses conversion_tags, finalize_spawn, mob_ai. No 26.1 → 1.21.11
behaviour change reaches the Rust — the one real version difference was
already implemented — but one pre-existing Rust-vs-vanilla gap in the
item's lay-spawn activity was fixed in passing, so the item counts as
changed.

`TryLaySpawnOnFluidNearLand` is gone in 1.21.11; FrogAi uses
`TryLaySpawnOnWaterNearLand`. Manual decompile diff of the two classes:
the support check changes from "fluid in `#supports_frogspawn` OR block
in `#supports_frogspawn`" to "fluid is exactly `Fluids.WATER`";
everything else identical (gates, collision face, above-air, place +
game event + sound, erase IS_PREGNANT). The Rust's `LaySpawn::lay`
already checks `fluid == Water` — no logic change, only the class-name
citations (2 spots). `TryFindLandNearWater` is identical in both
versions (verified by manual decompile diff); the Rust cited a
nonexistent `TryFindLandNearLiquid` in 3 spots — fixed (that class
exists in neither jar).

`FrogAttackablesSensor.isMatchingEntity` gains a
`!hasMemoryValue(HAS_HUNTING_COOLDOWN)` gate in 1.21.11, but a jar-wide
constant-pool scan shows nothing sets the memory on a frog: the only
1.21.11 references are the declaration, AxolotlAttackablesSensor,
FrogAttackablesSensor, AxolotlAi and Axolotl — the memory is an axolotl
thing, never present on a frog's brain, so the gate never blocks. No
Rust change. (`getMemoryToSet()` → `getMemory()` and the `requires()`
removal are sensor API changes with no Rust mirror.)

The brain-provider refactor (FrogAi/GoatAi `getActivities()` →
`makeBrain` + `addActivity*`; Frog `BRAIN_PROVIDER` → `SENSOR_TYPES` +
`MEMORY_TYPES` + `brainProvider()`) is API shape only: activity
contents, priorities, speeds and conditions verified field by field
against the Rust's goal encoding (CORE panic 2.0 + cooldown countdowns;
IDLE look/Breed/Tempt 1.25/ShootTongue/stroll/RunOne croak 3:1; SWIM;
LAY_SPAWN with Tempt blocked while pregnant; LONG_JUMP with the
TEMPTING_PLAYER/BREED_TARGET gates via goal priority; TONGUE erasing
the target on stop; GoatAi LONG_JUMP conditions via
`goat::long_jump_activity`). `Frog.canEat` and the goat ram conditions
are `TypedInstance`-removal rewrites with the same outcome
(`getType().is/equals`); `UniformInt.minInclusive()` → `getMinValue()`;
`canAttack(Mob)` → `canAttack(Frog)`. Croak, the three long-jump
behaviours, Frog's poses/walk/fall/travel/spawn/navigation are
identical modulo `level.getRandom()` → `level.random`. Method-name
diff run explicitly: only the expected refactor/rename differences.

`ShootTongue` (frog package, in no report item) is identical in both
versions; the Rust's goal matches it throughout (path-ends < 1.75
gate, 100-tick duration, tongue pull * 0.75, eat on the 7th CATCH tick
— the Rust's pre-increment + >6 equals vanilla's post-increment +
>=6, EAT ends at timer >= 10, unreachable list max 5 / 100 ticks; frog
attack damage is 10.0, so the unconditional RemoveKilled matches
vanilla's conditional one). The IS_TEMPTED carry-forward from item 38
is discharged for Frog/Goat: the jar-wide scan shows they reference
IS_TEMPTED only in their MEMORY_TYPES registration lists — no
frog/goat-side read exists, and neither Rust file reads a tempted
memory.

Fix: `land_near_water` checked `fluid == Water` for the block below the
neighbour, which is true for waterlogged blocks (`BlockFacts.fluid` is
Water when `waterlogged=true`); vanilla's
`getBlockState(below).is(Blocks.WATER)` is true only for actual water
blocks. Changed to a block-name check (`== "minecraft:water"` — source
and flowing are both that block). Also moved the `Frog.canEat` doc line
from above `walk_target` (wrong item) to above `can_eat`.

Pre-existing observations, not fixed: the Rust's unreachable-target
TTLs are per-entry while vanilla's `setMemoryWithExpiry` refreshes the
whole list's expiry on every add; `LaySpawn::lay` does not fire the
`GameEvent.BLOCK_PLACE` vanilla fires; the idle RunOne's full weight
distribution is approximated by the 3/4 croak roll (documented in the
header). All version-neutral.

Tests: all 9 frog tests pass
(`cargo test -p horizon-client-lib --lib --no-default-features
mobs::frog::`). `rustfmt --edition 2024 --check` clean on frog.rs.
Rank 2 now 220/492 (54 changed, 166 no-change).
work-items.json rank2.closed = 220 on 2026-11-20.

## Item 42 — horizon-client-render/gfx.rs (rank 2, CHANGED)

SectionRenderDispatcher$RenderSection, Options, DrawableGizmoPrimitives$Group,
BlockOutlineRenderState, FeatureRenderDispatcher, NativeImage,
DrawableGizmoPrimitives, DepthStencilState (removed), NativeImage$Format,
NativeImage$WriteCallback, OutlineBufferSource$EntityOutlineGenerator,
DrawableGizmoPrimitives$Line/$Point/$Quad/$TriangleFan,
WeightedList$Flat: 16 classes, 81 methods, 6 named. Harnesses
block_particles, blockstate_transform, breaking_model, mipmap,
model_baking, particle_providers, sprite_animation, tab_list. The one
real behaviour change is the removal of the `exclusiveFullscreen`
option; everything else in the item is identical, visibility-only, or
vanilla GPU plumbing the Rust does not mirror.

`Options.exclusiveFullscreen` is gone in 1.21.11 — field, accessor,
`initialExclusiveFullscreen`, both tooltips, `processDumpedOptions`
line. A jar-wide constant-pool scan finds ZERO references to
`exclusiveFullscreen` in the whole 1.21.11 client jar, and
`BackendOptions` (the record that carried the flag into
`RenderSystem.initBackendSystem` in 26.1) is gone too. The option never
controlled the window mode: `Window.setMode()` in BOTH versions always
calls `glfwSetWindowMonitor` with the monitor and the preferred video
mode — exclusive fullscreen — and in 26.1 the option was read once at
startup into the backend flag (changing it mid-game did nothing until
restart). The Rust's option → winit Exclusive-vs-Borderless mapping
mirrored 26.3 (`Window.exclusiveFullscreen`/`setExclusiveFullscreen`
exist in neither jar). So the option is removed in full and fullscreen
is now always the preferred-video-mode path, like vanilla: the
options-file default, the `Opt::ExclusiveFullscreen` variant, the Video
Settings row and the graphics-screen row, the `FullscreenState`
exclusive field/`set_exclusive`/reapply branch, and the `exclusive`
parameter of `fullscreen_request` are all gone; `fullscreen_request`
always takes the preferred mode if the monitor has it, else the current
mode (`Monitor.getPreferredVidMode`), falling back to borderless only
when no usable mode exists. `borderless_fullscreen` itself stays: the
HDR calibration window, the splitscreen-guest F11 and
`settle_created_fullscreen` use it, and those are Horizon paths, not
vanilla fullscreen. The `fullscreenResolution` preferred mode stays
(`preferredFullscreenVideoMode` exists in both jars' Window;
`VideoSettingsScreen.addOptions` is unchanged between versions).
User-visible: fullscreen now switches display mode like vanilla.

The removal's blast radius, all jar-verified: `VideoSettingsScreen.
displayOptions` drops the entry — that is item 733's single method, so
item 733 is DISCHARGED by this change. The `Opt::ExclusiveMode`
resolution row keeps its variant name but its caption key changes from
`options.fullscreen.exclusive.mode` (a key in NEITHER jar's lang — a
26.3-ism) to `options.fullscreen.resolution` ("Fullscreen Resolution"
in both jars' lang), and the `the_new_video_options_read_as_vanilla_
words_them` expectations change with it. The two graphics-screen hints
are reworded ("Fills the screen (borderless)." → "Takes the display
over, at the mode below."; "The display mode exclusive fullscreen
uses. …" → "The display mode fullscreen uses. …") and the three old
hint keys removed from all 87 lang files (the lang test requires every
locale key to exist in en_us.json, so the removals land together; the
reworded strings fall back to English elsewhere until re-translated).
`FramerateLimitTracker.getThrottleReason` is IDENTICAL in both jars and
has neither a focus nor an exclusive-fullscreen input, so the
`FrameContext` fields and the `(focused || !exclusive_fullscreen)` gate
in `throttle_reason` — a 26.3-ism — are removed; the game-loop harness
never exercises that branch (no (iconified=false, focused=false) limit
case; its scope note explicitly excludes it). FramerateLimitTracker is
in NO report item (report gap, recorded here).

No-change verdicts, all jar-verified: `RenderSection.getVisibility`
is byte-identical (`elapsed >= fade ? 1.0 : elapsed / fade`), and the
Rust's `chunk_visibility`/`fade_millis` match it (test pins it); the
section-dispatcher async/uber-buffer rewrite is vanilla GPU plumbing
with no Rust mirror. `BlockOutlineRenderState` is NOT new in 1.21.11 —
it moved packages (`state.level` → `state`) and is otherwise identical.
`FeatureRenderDispatcher` collapses its solid/translucent split into
one `renderAllFeatures` (shadows first); the Rust's
`PreparedFrame.executeTranslucent` citation named a class in neither
jar and now cites `renderAllFeatures`. `NativeImage.
computeTransparency`/`isClosed` are removed and `resizeSubRectTo`
switches STB filter (`nstbir_resize_uint8_linear` →
`nstbir_resize_uint8`), but the Rust mirrors neither method (the resize
comment cites the Rust's own `square_downscale` box average).
`DepthStencilState` is removed; 26.1's `DEFAULT` was (LESS_THAN_OR_
EQUAL, write=true) and 1.21.11's `RenderPipeline.Builder.build()`
defaults to exactly that (LEQUAL + write), so the two gfx.rs comments
now cite the builder default instead. `DrawableGizmoPrimitives` is
identical modulo names and the `CameraRenderState` package move; the
Rust's opaque-first grouping matches. `WeightedList$Flat` is a ctor
visibility change only — but the OUTER `WeightedList.isEmpty()` DID
change (`selector == null` → `items.isEmpty()`): that is item 137's
scope, referred, not fixed here. `Options.maxAnisotropyValue` is
unchanged (`sampler_anisotropy` matches); the `chatVisibility`/
`improvedTransparency` callback changes are Options$N anonymous classes
→ item 364. `keyDebugLightmapTexture`, `isSoundDeviceDefault`,
`getCloudStatus`→`getCloudsType`, `resizeGui`→`resizeDisplay` have no
Rust mirror. Item 117 (Window) owns the remaining `setMode`/
`updateFullscreen`/ctor differences — but NOT the fullscreen mode
itself, which this item settles as always-exclusive.

Tests: horizon-client-input lib tests 127 passed, 0 failed;
`cargo check -p horizon-client-render --lib --tests` and `cargo check
-p horizon-client --lib --tests` clean; `cargo check -p
horizon-client-lib --example client_probe` clean (covers the
game_loop_probe edit — the probe is a module of client_probe, not its
own cargo target); lang tests 16 passed incl.
`bundled_files_are_served_and_have_no_stale_or_repeated_keys`;
targeted render tests pass (`video_page_lists_vanilla_display_and_
quality_options_in_order`, `fullscreen_mode_picks_a_monitor_mode_or_
the_current_one`, `vanilla_video_rows_work_on_the_graphics_screen`,
`chunk_fade_is_vanillas_get_visibility`). `rustfmt --edition 2024
--check` clean on the edited leaf files (window.rs, options.rs,
startup_menu.rs, game_loop_probe.rs, options_file.rs, settings.rs);
gfx.rs, options_ui.rs and graphics_ui.rs are module roots and were
hand-checked against rustfmt's layout instead. The ignored
`the_new_video_options_read_as_vanilla_words_them` runs on CI with the
1.21.11 prepared lang data.
Rank 2 now 221/492 (55 changed, 166 no-change).
work-items.json rank2.closed = 221 on 2026-11-20.

## Item 43 — mobs/tame.rs (rank 2, CHANGED)

Cat, WolfModel, CatVariants, CatRenderer,
TamableAnimal$TamableAnimalPanicGoal, AdultFelineModel (removed),
BabyFelineModel (removed), BegGoal, CatSitOnBlockGoal,
Wolf$WolfAvoidEntityGoal, WolfVariants, BiomeCheck: 12 classes, 48
methods, 9 named. Harnesses entity_model, mob_ai. Three real behaviour
fixes: the unified 1.21.11 feline/wolf models, the wolf pup's sound
variant draw, and three cat goals that could never fire because they
gated on block tags that do not exist.

1.21.11 unifies the feline and wolf models. 26.1 has
`AbstractFelineModel` + `AdultFelineModel` / `BabyFelineModel` (with
Adult/Baby Cat/Ocelot subclasses) and an abstract `WolfModel` with
`shakeOffWater` + `setSittingPose` overrides in Adult/Baby subclasses;
1.21.11 has one `FelineModel` (+ thin `CatModel` with
`CAT_TRANSFORMER = scaling(0.8F)` and `OcelotModel`) and one concrete
`WolfModel` (with `real_head`, `upper_body`, `real_tail`,
`createMeshDefinition`, `BABY_TRANSFORMER`, the sitting pose inlined,
and the shake replaced by per-part rolls). Babies use the same poses at
ageScale 0.5. The Rust already implemented the 1.21.11 adult shapes but
kept the 26.1 baby branches: `feline()`'s baby sitting branch (body
xRot −0.43633232, head/tail offsets), baby lie-down branch, and the
`!baby` guard on the standing body xRot; `anim()`'s baby sitting body
xRot −π/2 and baby roll on the shared head/tail. All are removed: the
feline body xRot and the adult sitting/lie-down branches now run for
babies too (scaled by `age`), the wolf sitting pose is the unified
`upper_body` offset (y += 2.0 * age, xRot 2π/5), and the roll is the
unified guarded path (realHead 0.0, upperBody −0.08, body −0.16,
realTail −0.2). The `cat_anim` / `anim()` doc citations now name the
1.21.11 classes.

The `cats_can_lie_on` / `cats_can_sit_on` block tags do not exist in
either jar (only `beds.json` and item `cat_food.json`), and nothing
synthesizes them (`load_tag_lists` reads data files only), so the three
cat goals that gated on them could never fire: `can_lie_on`
(CatLieOnBedGoal), `can_sit_on` (CatSitOnBlockGoal), and
`CatRelaxOnOwnerGoal.canUse`. All three now use vanilla's hardcoded
checks, decompiled from both jars: lie-on is empty-above + a bed
(`BlockTags.BEDS`); sit-on is empty-above + an unopened chest, a lit
furnace, or a bed foot (the Rust's post-gate logic already implemented
exactly this — but its fallthrough arm relied on the tag gate to filter
to beds, so it now checks the `beds` tag explicitly, or every block
would be sittable); relax-on-owner checks the bed under the owner
(`ownerPosState.is(BlockTags.BEDS)`). CatLieOnBedGoal /
CatRelaxOnOwnerGoal / CatTemptGoal are in NO report item (unchanged
classes) — the tag gates are a 26.3-ism, fixed here because the goals
live in this item's file and classes. New test
`a_cat_sits_on_a_chest_and_a_bed_foot_and_lies_on_a_bed` pins the
sit/lie targets (chest yes, unlit furnace no, bed foot yes, air no;
lie-on bed yes, chest no); it needs the prepared data, so it
early-returns locally and runs on CI.

`Wolf.getBreedOffspring` draws a random sound variant in BOTH versions;
the Rust's breeding set every pup's `sound_variant` to None ("keeps the
default `classic`"). Wolf pups now draw from the mother's random, in the
same shape as the axolotl/mooshroom draws in that block; cats correctly
keep None (no cat sound variant in 1.21.11). The wolf sound variant is
also now synced: index 23 is decoded in `remote_mob_data.rs`
(`minecraft:wolf_sound_variant` → `m.extra.sound_variant`) and sent in
`lan.rs` (classic by default), where before it was neither. The entity
indices were javap-verified: Cat variant 19, lying 20, relax 21, collar
22; Wolf interested 19, collar 20, anger 21, variant 22, sound 23.

No-change verdicts, all jar-verified: the cat sound variant is GONE in
1.21.11 (no `CatSoundVariant` class, no `DATA_SOUND_VARIANT_ID`, no
`sound_variant` NBT, no `getSoundSet`; all cat sounds are fixed
`SoundEvents.CAT_*`) and the Rust already matches (`sound_variants`
returns `[]` for non-wolf; `entity_nbt.rs` writes no cat sound
variant). The wolf sound variant is KEPT but flattened (26.1: adult/baby
`WolfSoundSet` records; 1.21.11: flat 6 holders, no step, no baby/adult
split; step is fixed `SoundEvents.WOLF_STEP`); the Rust's 7 names match
the 1.21.11 registry and `variant_sound` builds
`entity.wolf[_variant].kind` with step shared — only its doc was stale
("Babies use the baby set"), now fixed, along with the `spawning.rs`
comment (wolves only) and two `TamableAnimal.feed` citations in
`creatures.rs` (the method is gone in 1.21.11, inlined at the call
sites: wolf heals 2× nutrition, cat 1× — the Rust already computes
both). Cat/wolf textures and variant registries match (bare
`entity/cat/tabby`, one texture per variant; wolf 9 names/biomes, one
`AssetInfo`). Dye is the DyeItem shape in the Rust already. Cat goals
1–12 + targets, wolf goals 1–10 + targets 1–8, BegGoal, CatTemptGoal,
Cat.handleLieDown amounts and CatRelaxOnOwnerGoal all match vanilla.

Two things recorded, not changed. The cat spawn variant: vanilla's
`SpawnPrioritySelectors` force all-black in a swamp hut
(`CATS_SPAWN_AS_BLACK`) and weight it at moon brightness ≥ 0.9; the
Rust's uniform 0..10 (0..11 on a full moon) matches the moon cases
exactly, and only the swamp-hut case differs — `BiomeFacts` has no
structure check, and the swamp biome is not a proxy for the hut, so this
stays a documented approximation. The entity-data serializer table:
1.21.11's `EntityDataSerializers` registration order drops the
cat/cow/pig/chicken sound serializers and renumbers from COW_VARIANT
down (COW 22, WOLF 23, WOLF_SOUND 24, FROG 25, PIG 26, CHICKEN 27,
ZOMBIE_NAUTILUS 28, … HUMANOID_ARM 38); the Rust's `entity_data.rs`
table is the 26.1 numbering by design and the adapter's `V1_21_11`
remap was verified run-by-run against the jar-derived order — it matches
exactly, so no re-base is needed. Item 335 owns `network/syncher`; its
work is verification of that remap, not a rewrite. The wolf ctor path
malus (`ON_TOP_OF_POWDER_SNOW` → `DANGER_POWDER_SNOW`) has no Rust
mirror, the same gap as the fox's in item 33.

Tests: `cargo check -p horizon-client --lib --tests`, `--bin
horizon-client --tests` and `cargo check -p horizon-client-lib --lib
--tests` all clean (the last validates the new tame test compiles);
remote_mob_data tests 26 passed, 0 failed, 2 ignored; lan tests 13
passed, 0 failed, 17 ignored; creatures tests 14 passed, 0 failed, 22
ignored (all from the freshly built horizon-client lib test binary).
The horizon-client-lib test binary link OOMs on this VM at JOBS=1 under
3-stream load (the known local constraint) — the tame tests, including
the new sit/lie test, run on CI. `rustfmt --edition 2024` clean on the
edited leaf files (tame.rs, spawning.rs, remote_mob_data.rs); lan.rs and
finalize.rs `--check` clean (module roots, hand-checked); creatures.rs
has pre-existing drift at the wolf-collar-dye line, untouched.
Rank 2 now 222/492 (56 changed, 166 no-change).
work-items.json rank2.closed = 222 on 2026-11-20.

## Item 44 — game/commands.rs (rank 2, CHANGED)

TimeCommand, Item, ItemPredicateArgument$Context/$ComponentWrapper/
$PredicateWrapper/ItemPredicateArgument, LevelData, MessageArgument/
$Message, BlockPredicateArgument/$BlockPredicate/$TagPredicate,
BlockStateArgument, BlockPosArgument, Commands, Commands$2$1,
AdvancementCommands$Action/$1/$2/$Mode, ExperienceCommand$Type,
FillCommand/$1UpdatedPosition, TimeCommand$ClockGetter (removed): 24
classes, 77 methods, 6 named. Harnesses arrow_mob_permission,
command_parse, container_component_names, mob_ai,
mob_rider_navigation, redstone_catalog, sculk, trading. The real
change is /time: 26.1's multi-clock command is replaced by the classic
1.21.11 command, and /swing (a 26.x command) is removed.

26.1's TimeCommand is the WorldClock system: `addClockNodes` (set
<time> / set <timemarker> / add / pause / resume / rate / query time /
query <timeline> [/ repetition]), `time of <clock>`, `getDefaultClock`,
`ServerClockManager`, error keys `commands.time.no_default_clock` /
`no_time_marker_found` / `wrong_timeline_for_clock`. 1.21.11 has the
classic command, and the Rust's `cmd_time` and the tree (`clock_nodes`
in horizon-command/src/vanilla.rs) now implement exactly that: `set
day|noon|night|midnight` (1000/6000/13000/18000) or `set <time>`, `add
<time>` (min 0 — `TimeArgument.time()` is `new TimeArgument(0)`,
javap-verified; 26.1's add allowed negatives), `query
daytime|gametime|day`. Set/add apply to every level and force a time
sync; the message is `commands.time.set` ("Set the time to %s") with
the set value (set) or the new day time (add), and the return is the
source level's day time % 24000. Queries use the single key
`commands.time.query` ("The time is %s"): daytime % 24000, gametime %
2147483647, day count (`getDayTime() / 24000`, javap-verified) %
2147483647. No already-at-time check, no add clamp, no pause/rate/
marker/timeline. The old keys (`commands.time.set.absolute`,
`query.absolute`, `query.gametime`, `pause`, `rate`, `no_default_clock`,
`no_time_marker_found`, `wrong_timeline_for_clock`) do not exist in
1.21.11's lang at all (verified against the jar's en_us.json) — the old
command's feedback would have rendered as raw keys. The all-levels
semantics come free: the Rust keeps one canonical day time shared
across levels (`enter_level` copies it into a parked level on swap),
and the hub's `sync_world` broadcasts the jump on the next tick. The
native `time_skip` hook is kept for set/add (set via `jump_clock`,
which also restarts the clock partial). The shared WorldClock
infrastructure (`ui.clock`, `ui.command_clocks`, `ui.timelines`,
`command_clock_*` helpers, `jump_clock`) stays — other items own it,
and the native-mod `set_day_time` API still uses the helpers. The
QUIET list drops the four clock query keys for the single
`commands.time.query`.

/swing is gone in 1.21.11 (`Commands` drops
`SwingCommand.register`): the tree (tree_world.rs), the dispatch, and
`cmd_swing` are removed, with the two horizon-command tree tests. The
/swing ANIMATION packet/echo is separate and stays — only the command
goes. Removing `time.set.marker` orphaned `TimeMarker` (dead-code
warnings, which CI's clippy -D warnings would fail on), so the struct,
`UiState.markers`, `ResourceIds.markers`, the marker loop in
`overworld_timelines`, and both load sites go too; the timelines half
stays (still loaded into `reg.timelines`), and the marker test becomes
timelines-only. `reg.time_markers`, `Suggests::TimeMarkers`, and the
`ArgType::Id("time_markers")` arm stay: pub API, no warnings, inert.

No-change verdicts, all jar-verified. FillCommand is identical except
the area computation: 26.1 uses `long`, 1.21.11 uses `int`, which wraps
for regions over 2^31 blocks and bypasses the too-big check; the Rust
keeps the i64 (26.1) form deliberately — strictly safer, differing only
on regions no loaded area can span in practice. MessageArgument,
BlockPredicateArgument, BlockStateArgument, BlockPosArgument,
ItemPredicateArgument, AdvancementCommands, ExperienceCommand$Type
(POINTS + LEVELS in both) are parameter-name/visibility noise only.
`Item.getDefaultMaxStackSize()` reads the same component with the same
default 1 and `getName(ItemStack)` is identical (the component-storage
rewrite is horizon-items' scope; 1.21.11's added no-arg `getName()` is
noted for that item). LevelData GAINS abstract `getDayTime()` /
`isThundering()` / `isRaining()` / `setRaining(boolean)` in 1.21.11 plus
a "Level time" crash detail — save/world-data scope, not commands.rs.
The item-31 referral is DISCHARGED: 26.1's `onGameRuleChanged` has an
ADVANCE_TIME branch (broadcast the full clock sync) that 1.21.11 lacks,
and the Rust has no such hook to remove (`set_game_rule` just sets +
syncs). `advance_time` / `advance_weather` are the rule names in BOTH
jars — no rename needed.

Tests: horizon-command lib tests 52 passed, 0 failed (incl. the updated
/time parse, suggestion and min-0 tests and the world-command trees
without /swing); `cargo check -p horizon-client --lib --tests` clean
with ZERO warnings (validates the cmd_time rewrite, the cmd_swing
removal, the TimeMarker removal and the rewritten timelines test
compile). The horizon-client lib test binary link OOMs on this VM at
JOBS=1 under 3-stream load (the known local constraint) — the chat and
commands tests run on CI. `rustfmt --edition 2024` clean on the edited
leaf files (vanilla.rs, tree_world.rs, tests.rs, world_cmds.rs,
admin.rs); commands.rs `--check` clean; chat.rs has pre-existing drift
at lines 6, 947, 2397, untouched.
Rank 2 now 223/492 (57 changed, 166 no-change).
work-items.json rank2.closed = 223 on 2026-11-20.

## Item 45 — lan/hub.rs (rank 2, NO-CHANGE)

ChunkMap, PlayerChunkSender: 2 classes, 31 methods, 10 named. Harnesses
chest_load, container_title, datafix, falling_block, natural_spawn,
player_sync, portal_tickets, spawner. Every named method is
behaviour-identical between the jars; no Rust change.

The javap diff is, method by method: the ctor's field-initialization
order + visibility (private → package) + the param rename
`Supplier<SavedDataStorage>` → `Supplier<DimensionDataStorage>`;
ChunkPos API renames (`pack()` → `toLong()`, `pack(x,z)` →
`asLong(x,z)`, `unpack(long)` → `new ChunkPos(long)`, record accessors
`x()`/`z()` → public fields — 26.1 made ChunkPos a record, 1.21.11 has
it as a plain class); and `printFuture`, identical. `tick`,
`processUnloads`, `saveChunksEagerly`, `resendBiomesForChunks`,
`getPlayers`, `getPlayerViewDistance`,
`anyPlayerCloseEnoughForSpawning`, `collectSpawningChunks`,
`updatePlayerStatus`, `move`, `updateChunkTracking`,
`applyChunkTrackingView`, `isChunkTracked`,
`isChunkOnTrackedBorder`, `waitForLightBeforeSending`, and the whole of
PlayerChunkSender are byte-identical.

The one real difference is chunk-IO scope, not hub.rs:
`SimpleRegionStorage.upgradeChunkTag` (the multi-arg overload lives in
the parent class) — 26.1 upgrades with `version >= targetVersion` as
the early-out, while 1.21.11 uses `version == currentVersion` and
applies the legacy structure-data fixer
(`LegacyStructureDataHandler`) before datafixing. Referred to the
chunk_io items (197/214/241/262/651). `getChunkDataFixContextTag`'s
generator param type changes with identical output.

The hub.rs mirrors were all verified matching: `guest_view` is
`Mth.clamp(requested, 2, server)`; `update_view` sends
SetChunkCacheCenter on a centre change, forgets chunks that left the
view, and `requeue` drops pending out-of-view chunks without a forget
packet (PlayerChunkSender.dropChunk sends ForgetLevelChunk only for
sent chunks); `next_batch`/`update_chunks` reproduce the sendNextChunks
quota math, batch framing and closest-first order; `resend_biomes` is
per-player, sent-only, one packet; the light-broadcast border check is
`isChunkOnTrackedBorder`; `tracked` is ChunkTrackingView.contains; and
`close_enough_for_spawning` is the chunk-centre `< 16384.0` check.
(The carry-forward listed item 45 among the hub.rs items for per-level
time-sync content, but ChunkMap/PlayerChunkSender contain no time
content — that belongs to the other hub items.)

Tests: no code changed, no build needed.
Rank 2 now 224/492 (57 changed, 167 no-change).
work-items.json rank2.closed = 224 on 2026-11-20.

## Item 46 — banner.rs (rank 2, CHANGED)

Sheets, LoomMenu: 2 classes, 48 methods, 8 named. Harnesses cauldron,
menu_click, place_recipe, trading. The banner/shield cloth textures
moved in 26.x: 1.21.11 keeps them at the top of `textures/entity/`
(`entity/banner_base.png`, `entity/shield_base.png`,
`entity/shield_base_nopattern.png`), 26.1 moved them under the pattern
subdirectories (`entity/banner/banner_base.png`, ...). Same bytes in
both jars (sha256-verified); only the path changed. The holding
constants moved too: banner cloth is 26.1 `Sheets.BANNER_BASE` but
1.21.11 `ModelBakery.BANNER_BASE` (1.21.11's `Sheets.BANNER_BASE` is
"base", the base pattern layer); shield is 26.1
`Sheets.SHIELD_BASE`/`SHIELD_BASE_NO_PATTERN` but 1.21.11
`ModelBakery.SHIELD_BASE`/`NO_PATTERN_SHIELD`. The draw sequence is
identical in both versions (javap-verified on BannerRenderer and
ShieldSpecialRenderer): cloth white, base layer dyed, up to 16 pattern
layers; shield picks base vs nopattern on `has_patterns`.

Fixed 17 sites: `entity/banner/banner_base` → `entity/banner_base`
(block_entity_render.rs x4, block_entity_render_tests.rs x2,
mob_blocks.rs, item_render/special.rs x3) and
`entity/shield/shield_base` → `entity/shield_base` (special.rs x7,
including nopattern). Citations fixed: `getBannerSprite`/
`getShieldSprite` → `getBannerMaterial`/`getShieldMaterial`
(banner.rs, game/banners.rs — same computeIfAbsent-on-assetId logic);
`cutoutBlockItemSheet` → `cutoutBlockSheet` (mob_blocks.rs — 1.21.11
has no cutoutBlockItemSheet). `block_entity_cem.rs`'s "banner_base"
strings are OptiFine CEM part names, not textures — untouched. Chest
materials differ only in API shape (same texture file names in both
jars); bed/shulker/sign/decorated-pot/armor-trim logic identical.

LoomMenu was already 1.21.11-shaped and needed no change, verified
against the decompile and the 1.21.11 datagen: the dye slot takes any
DyeItem (no `#loom_dyes` tag, no DYE component — the datagen has no
`minecraft:dye` component on any item, so the name fallback is the live
path); the pattern slot takes any stack with PROVIDES_BANNER_PATTERNS
(no `#loom_patterns` tag); the component is a TagKey in 1.21.11
(`#minecraft:pattern_item/flower`, 10 items) resolved through the
registry, a HolderSet in 26.1 — the Rust resolves the tag; menu.rs's
slotsChanged/clickMenuButton/quickMoveStack match vanilla exactly.
Version-neutral divergence, documented not fixed: vanilla's
slotsChanged has no else branch (it does nothing when banner or dye is
empty), while the Rust's `loom_changed` clears the result, patterns and
index — the engine's full-recompute shape; same in both jars.

Tests: `cargo check -p horizon-client-render --lib --tests
--no-default-features`, `cargo check -p horizon-client --lib --tests
--no-default-features` and `--bin horizon-client --tests` all clean;
horizon-client-render banner tests 3 passed, 0 failed. The
horizon-client lib test binary link OOMs on this VM under 3-stream
load (established) — its tests run on CI. rustfmt clean on edited
files except pre-existing drift at special.rs:449 (book renderer,
untouched).
Rank 2 now 225/492 (58 changed, 167 no-change).
work-items.json rank2.closed = 225 on 2026-11-20.

## Item 47 — mobs/bee.rs (rank 2, CHANGED)

Bee, NeutralMob, ResetUniversalAngerTargetGoal: 16 classes, 38 methods,
9 named. Harness mob_ai. The Bee class itself is identical between the
jars (goals, save data, sting poison 10 s normal / 18 s hard, nectar
drip, `wantsToEnterHive`, `attractsBees`) except the `PathType`
rename `FIRE_IN_NEIGHBOR` → `DANGER_FIRE`; `ResetUniversalAngerTargetGoal`
differs only in `is(EntityType.PLAYER)` → `getType() == EntityType.PLAYER`.
The real changes are all in `NeutralMob.updatePersistentAnger`, shared
by every neutral mob (all call it with `stayAngryIfTargetPresent = true`
except the bee, `false` — javap-verified in both jars):

- Timer restart: 1.21.11 restarts the timer unconditionally whenever a
  target is held; 26.1 restarts only on a new target or when
  `stayAngryIfTargetPresent`. Only the bee differs (its anger expired
  after 20–39 s even with the target held in 26.1; in 1.21.11 it never
  expires while the target is held).
- No immediate forgiveness: 26.1 stops anger at once when the persistent
  target is a creative/spectator player or the difficulty is Peaceful;
  1.21.11 has no such branch — the goal layer drops a creative/spectator
  player from the target and only then does the timer run out, and on
  Peaceful neutral mobs stay angry (and keep attacking).
- `isValidPlayerTarget`: 26.1 adds a Peaceful check; 1.21.11 checks only
  creative/spectator (it gates only the universal-anger arm of
  `isAngryAt`, and both Rust mirrors already omit the difficulty check).
- `Mob.getTarget()`: 26.1 filters through `asValidTarget` (null for a
  creative/spectator or unattackable target) with a raw
  `getTargetUnchecked()`; 1.21.11's `getTarget()` is the raw field.

Changes: bee.rs restarts the timer every tick a target is held and drops
the immediate stop; tame.rs (wolf + iron golem) drops the 26.1 `getTarget`
filter and the immediate stop, simplifying the stop condition to
`angry_at.is_some() && anger == 0`; polar_bear.rs drops the immediate
creative/spectator forgiveness. Both now-unused `valid_player_target`
helpers (which carried the Peaceful check) removed. Also fixed the bee's
fire malus, which was on the wrong path type in both versions' terms:
`DamageFire` (in fire) instead of vanilla's `DANGER_FIRE` (near fire) —
the Rust's evaluator distinguishes them, so a bee pathed next to fire
at cost 8 instead of refusing it. Tests: the Peaceful test pinned the
26.1 behaviour — replaced by `peaceful_does_not_end_a_bees_anger_at_the_player`,
`a_held_target_restarts_a_bees_anger_timer` and
`a_creative_or_spectator_player_is_not_forgiven_at_once`.

Recorded, not changed (version-neutral, other files' scope): the
zombified piglin starts its anger only on a new target and counts down
even with a target held, though vanilla calls `updatePersistentAnger`
with `true` in both versions; the enderman has no `updatePersistentAnger`
mirror at all; the polar bear's update has no dead-mob-target branch.

Tests: `cargo check -p horizon-client --lib --tests
--no-default-features` and `-p horizon-client-lib --lib --tests
--no-default-features` clean, zero warnings; horizon-client-lib tests:
bee 6 passed, tame 17 passed, golem 10 passed, polar_bear 3 passed, 0
failed. rustfmt clean on bee.rs, tame.rs, polar_bear.rs.
Rank 2 now 226/492 (59 changed, 167 no-change).
work-items.json rank2.closed = 226 on 2026-11-20.

## Item 48 — game/dragon.rs (rank 2, CHANGED)

EnderDragon, EnderDragonRenderer, DragonFireballRenderer, EndCrystal,
TargetingConditions, DragonHoldingPatternPhase, DragonStrafePlayerPhase,
EndCrystalItem, Explosion, DragonRespawnStage (+5 inner),
EndSpikeFeature (+2 inner), DragonFlightHistory$Sample: 19 classes, 37
methods, 9 named. Harnesses dragon_contacts, dragon_render, entity_model,
mob_ai. 26.1's `DragonRespawnStage`/`EndSpikeFeature` are 1.21.11's
`DragonRespawnAnimation`/`SpikeFeature` (items 763/761 list the 1.21.11-side
classes; the fight class itself is item 483's scope).

The dying render is rewritten. 26.1 draws the dying dragon in one pass,
`entityCutoutDissolve` with colour `ARGB.white(1 - deathTime/200)` and no
overlay — the dragon dissolves starting from the explosion texture's opaque
regions. 1.21.11 draws two passes: order(0) `dragonExplosionAlpha`, the
explosion texture unlit and fog-free, alpha-tested by
`ARGB.white(deathTime/200)` (keeps texels at or above the threshold, so the
dragon dissolves starting from the explosion texture's transparent regions —
the opposite pattern); order(1) `entityDecal`, the skin lit with the hurt
overlay, drawn only where the first pass wrote depth (EQUAL depth test).
A skin-only pass cannot reproduce it: 8622 of the dragon model's 34272 mapped
texels are transparent (the wing membranes are shaped by the cutout, verified
against the decompiled EnderDragonModel UVs and the jar's dragon.png), and
where the skin is transparent the explosion texture shows through. The Rust
now emits both passes in material 0 (emission order preserved, GreaterEqual
depth test): new overlay 46 `DRAGON_EXPLOSION_OVERLAY` (unlit, fog-exempt,
`texel.a < tint.a` discard, no 0.1 cutout, normals zeroed — the alpha shader
has no lighting at all), overlay 38 `DISSOLVE_OVERLAY` reworked to the
1.21.11 test direction (`mask.a < tint.a` discard — the mask test stands in
for the EQUAL depth test, which this engine has no pipeline for), and new
overlay 47 `DISSOLVE_HURT_OVERLAY` for the hurt-red flash during the first 10
ticks of death. `dissolve_color` flips to `ARGB.white(death/200)`;
`dissolve_mask` is unchanged.

`EnderDragon.knockback` (the sitting guard) is removed in 1.21.11
(javap-verified: no override remains) — the dragon takes full
`LivingEntity.knockback` in every phase, perched included. The Rust drops the
guard; the test is rewritten (`a_melee_hit_knocks_a_perched_dragon_too`).
`tickDeath`'s final block runs at `dragonDeathTime == 200` in 1.21.11, not
`>= 200` (26.1); the Rust now uses `== 200` (death_time increments by exactly
1 per tick, so it cannot skip 200).

No change: the respawn stages (identical bodies; 1.21.11's `tick` takes an
extra unused BlockPos), the spike feature (identical `place`; only
`getSpikesForLevel`'s RNG differs — Xoroshiro in 26.1, LegacyRandom in
1.21.11 — and `end_spike.rs` already uses LegacyRandom), the fireball
(camera-facing quad, never culled), the crystal beam (both cutout 0.1, no
blend, lightmapped), the death rays (Rust already uses LegacyRandom 432, the
1.21.11 factory; its single draw equals vanilla's colour-only + depth-only
pair), TargetingConditions (the Rust already refuses Ghasts and checks
Peaceful, matching 1.21.11's added `canAttackType` arm), and all of
EnderDragon gameplay, EndCrystal, EndCrystalItem, Explosion and the two
phases. Citations fixed: `DragonRespawnStage` -> `DragonRespawnAnimation`
(3 sites), `EndSpikeFeature` -> `SpikeFeature` (2 sites).
`EnderDragonFight` -> `EndDragonFight` citations and the `alive_crystals`
field name are left to item 483 (the fight class's scope).

Correction to item 47's note: on Peaceful, neutral mobs stay angry in 1.21.11
(no immediate stop in `updatePersistentAnger`) but do NOT keep attacking —
`LivingEntity.canAttack` returns false for a player target on Peaceful in
both versions (javap-verified), so the target goal drops the player and the
anger then runs out with the timer.

Tests: `cargo check -p horizon-client-render --lib --tests
--no-default-features` and `-p horizon-client --lib --tests
--no-default-features` clean; WGSL validated by the
`shaders_translate_to_hlsl_for_directx_12_and_glsl_for_opengl` test (naga
parse + validate + HLSL/GLSL translation of entity.wgsl). rustfmt clean on
all touched files. The horizon-client lib test binary link OOMs on this VM
under 3-stream load (established) — the dragon tests run on CI.
Rank 2 now 227/492 (60 changed, 167 no-change).
work-items.json rank2.closed = 227 on 2026-11-20.

## Item 49 — game/minecarts.rs (rank 2, CHANGED)

AbstractMinecartRenderer, TntMinecartRenderer, AbstractMinecart,
NewMinecartBehavior, AbstractMinecartContainer, Minecart, MinecartChest,
MinecartCommandBlock, OldMinecartBehavior, MinecartSpawner$1,
NewMinecartBehavior$TrackIteration, NewMinecartBehavior$MinecartStep: 12
classes, 35 methods, 9 named. Two changes:

- The minecart texture moved in 26.x: 26.1
  `textures/entity/minecart/minecart.png` -> 1.21.11
  `textures/entity/minecart.png` (bytes identical, sha256
  eeb180e95bc10562 in both jars). The Rust's single sprite lookup in
  `cart_geometry` is fixed to `"entity/minecart"` — with the 26.1 path
  the lookup fails and no cart renders at all.
- The TNT flash phase used the raw int fuse; vanilla uses
  `(int)fuseRemainingInTicks` where `fuseRemainingInTicks = fuse -
  partialTicks + 1`, which differs at an exact tick boundary (alpha = 0,
  vanilla reads `fuse + 1`). One interpolated `fuse` now feeds both the
  swell and the flash. Same code in both jars — a version-neutral fix in
  this item's scope, under the item-47 precedent.

No change: the display-block gate (26.1 `!displayBlockModel.isEmpty()` vs
1.21.11 `getRenderShape() != INVISIBLE`) is unobservable — an empty model
bakes to nothing either way, and the culling box is `!isAir()` in both
versions; the model submit resolves to `entityCutoutNoCull` in both
(`EntityModel` injects it as the render-type function, javap-verified);
`interact` drops its Vec3 parameter in 1.21.11 but the bodies are identical
and the Rust never used a hit position; `tick`/`moveAlongTrack` (old and
new)/`getRedstoneDirection` differ only in the `BlockState.is` overload and
the `updateFluidInteraction` rename; the TNT swell, the container slowdown
(`container_keep` matches exactly) and all renderer pose math are identical.
MinecartFurnace is item 268; MinecartTNT/MinecartHopper are unchanged (no
report item); `MinecartCommandBase` is item 686.

Tests: `cargo check -p horizon-client --lib --tests --no-default-features`
clean; rustfmt clean. No geometry test covers the TNT flash path; the
horizon-client lib test binary link OOMs on this VM under 3-stream load
(established) — tests run on CI.
Rank 2 now 228/492 (61 changed, 167 no-change).
work-items.json rank2.closed = 228 on 2026-11-20.

## Item 50 — crates/horizon-items/src/items.rs (rank 2, NO-CHANGE)

EnchantRandomlyFunction, EnchantWithLevelsFunction (+$Builders),
LootCommand, SetItemDamageFunction, BonusLevelTableCondition,
UniformGenerator, LootPool (+$Builder), CopyBlockState (+$Builder),
ExplosionCondition, LootItemEntityPropertyCondition, DynamicLoot,
ApplyExplosionDecay, SetPotionFunction, FlintAndSteelItem, Tool$Rule: 19
classes, 82 methods, 4 named. One real difference, jar-verified — and it
needs no Rust change:

26.1's two enchant functions carry `includeAdditionalCostComponent`
(field, codec entry, builder method): when set, they write
`DataComponents.ADDITIONAL_TRADE_COST` — `2 + nextInt(5 + level*10) +
3*level` for EnchantRandomlyFunction, the enchantment cost itself for
EnchantWithLevelsFunction. 1.21.11 removes all of it (`enchantItem`
becomes static). Neither Rust loot evaluator ever implemented the
component — loot.rs and items.rs are already 1.21.11-shaped, and 0 of
the 1.21.11 jar's 1324 loot tables use the flag (25 use the two
functions). In 26.1 the flag appears only in `villager_trade/` data (21
files), never in loot tables.

`merchant.rs` keeps its flag handling: it is the trade-data mechanism
that reproduces the 1.21.11 hardcoded trade costs, verified against the
decompiled 1.21.11 classes — EnchantBookForEmeralds' `2 + nextInt(5 +
level*10) + 3*level`, doubled for `DOUBLE_TRADE_PRICE`, capped at 64,
and EnchantedItemForEmeralds' `min(base + levels, 64)`. Removing it
would break both trades. The 1.21.11 trade DATA that would use it does
not exist yet (1.21.11 has no villager_trade/trade_set data; trades are
hardcoded) — generating it is item 751's scope, which owns the
VillagerTrades listing classes. Notes for 751: the Rust reads only the
singular `given_item_modifier`, while 26.1 data uses the plural; the
book trade's 4-arg level bounds and empty-tag fallback have no Rust
mirror (unobservable with the standard trade).

No-change verdicts (all jar-verified): LootCommand only moves to
`createItemStack(1, false)` — the boolean is `checkOverstacked`, and
1.21.11 never validates (item 35); the Rust's `tool_stack` already
builds the stack directly with no validation. FlintAndSteelItem.useOn
is byte-identical in both versions and the Rust's game.rs arm matches
both branches (light a block / place fire). Tool$Rule identical.
LootPool's roll logic identical (only `validate` rewritten); pool math
verified in both evaluators. DynamicLoot, ApplyExplosionDecay,
CopyBlockState, SetItemDamageFunction, SetPotionFunction,
BonusLevelTableCondition, ExplosionCondition,
LootItemEntityPropertyCondition, UniformGenerator: bodies identical,
only MAP_CODEC → CODEC / `codec()` → `getType()` / `validate` →
`getReferencedContextParams` plumbing; the Rust mirrors are exact
(`survives_explosion`, `table_bonus`, `explosion_decay`, `copy_state`,
`set_damage`, `uniform` in both evaluators). Enchant level min is 1 in
both versions (no `min_level` in any enchantment JSON), so the Rust's
`Mth.nextInt(random, 1, max)` is exactly vanilla.

Recorded gaps (all version-neutral, vanilla-unobservable, not fixed):
the loot.rs evaluator lacks the five block-only kinds
(`survives_explosion`, `table_bonus`, `explosion_decay`, `copy_state`,
`dynamic`) — it only rolls entity/chest/fishing/vault/trial tables,
and all vanilla uses are in `blocks/` tables, rolled by items.rs,
which has all five; the items.rs evaluator omits the `bonus_rolls`
term — every `bonus_rolls` in 1.21.11 data is `0.0`; items.rs `choose`
keeps zero-weight entries as candidates — 0 weight-0 entries in
vanilla data; items.rs `entity_properties` answers false for non-`this`
targets (documented approximation).

No Rust changes; no build needed (no-change closure, item-45 precedent).
Rank 2 now 229/492 (61 changed, 168 no-change).
work-items.json rank2.closed = 229 on 2026-11-20.

## Item 51 — crates/horizon-client/src/game/raids.rs (rank 2, CHANGED)

VillageSiege (named findRandomSpawnPos, tick, trySpawn,
tryToSetupSiege), PoiManager (named getInSquare, isVillageCenter),
Raider (named aiStep, die, finalizeSpawn), Raids, ClockTimeMarkers
(**26.1-only — absent in 1.21.11**), ResetRaidStatus, SetRaidStatus,
PoiManager$DistanceTracker, PatrolSpawner (named tick),
Raids$RaidWithId: 10 classes, 18 methods, 10 named. Three changes,
all in raids.rs (+9/-6):

1. **VillageSiege now ticks in every dimension.** In both versions
`MinecraftServer` holds one `VillageSiege` in its custom-spawners
list (constant-pool scan: the only referrer in both jars) and every
`ServerLevel.tick` ticks all custom spawners — so the siege runs in
the Nether and the End too. The gate is `!isBrightOutside() &&
spawnEnemies`, and `isBrightOutside()` is `!hasFixedTime &&
skyDarken < 4` (identical bytecode in both jars) — never bright
where time is fixed. The only biome exclusion is
`without_zombie_sieges` = `mushroom_fields` only (jar-verified in
both versions). The Rust gated `tick_village_siege` to
`Dim::Overworld` with a bare `sky_darken() < 4` gate; removed the
gate and used the `isBrightOutside` formula (the same formula
`PatrolLevel.bright_outside` already uses). The engine ticks every
level per server tick and the siege state is per-level, so each
dimension gets its own siege — vanilla shares one instance across
levels (a quirk, not reproduced). Version-neutral fix under the
item-47 precedent.
2. **`raid_boss_at` lost its Overworld gate.** A raid can start in
the End: `CAN_START_RAID` defaults true and the End dimension type
carries no entry (jar-verified); only the Nether forbids raids,
via `can_start_raid: false` — which `create_or_extend_raid`
already honours through the attribute with no dimension gate of
its own. An End raid could exist and tick but its boss bar was
hidden. Version-neutral fix.
3. **Citation fix:** the roll comment cited
`ClockTimeMarkers.ROLL_VILLAGE_SIEGE`, a 26.1-only class. The
1.21.11 roll is `getDayTime() % 24000 == 18000`; the Rust already
used that form.

Verified, no change: the siege roll (26.1 uses the clock marker,
1.21.11 the day-time form — Rust already 1.21.11-shaped), setup /
spawn / find-spawn-pos (exact mirrors, WORLD_SURFACE height),
PatrolSpawner.tick identical in both versions with the day-5 gate
in timeline data (`early_game.json`: `can_pillager_patrol_spawn`
false at 0, true at 120000, modifier "and"; `#minecraft:
in_overworld` includes `early_game` — the Rust's
`GameplayAttributes` resolves it all), the patrol leader's
`findPatrolTarget` before placement (target ±500 of the origin —
the Rust reproduces it), Raider.aiStep's every-20-ticks join-raid
logic and no-action reset (exact mirrors), the brain's
SetRaidStatus/ResetRaidStatus 1-in-20 rolls and branches, PoiManager
/ DistanceTracker (Rust: Chebyshev distance capped at 7;
`village_centers` derives centres from villagers' remembered
POIs — version-neutral), Raids codec `{raids, next_id, tick}`.
1.21.11 adds `TYPE_END`/`getType` (`raids_end` file) and makes
`RAID_FILE_ID` a String; no save-path change — the engine saves
in its own `dimensions/<dim>/data/minecraft/raids.dat` layout and
`migrate_legacy` never moves `data/` files, so vanilla raid data
is unread either way and the per-dimension folders make
`raids_end` unnecessary. (The data/-file migration gap is
save/world-data scope, version-neutral.)

`cargo check -p horizon-client --lib --tests --no-default-features`
and `--bin horizon-client --tests` both clean (JOBS=1). No new
tests; raids.rs declares a file mod, so no rustfmt run.
Rank 2 now 230/492 (62 changed, 168 no-change).
work-items.json rank2.closed = 230 on 2026-11-20.

## Item 52 — client/resources/model (rank 2, CHANGED)

ModelManager (20), ResolvedModel (20), Material (16),
ModelDiscovery$ModelWrapper (14), ModelManager$ReloadState (10),
ModelManager$1 (8), ModelBaker (6), UnbakedModel (6), ModelState
(4), SpriteGetter (4), UnbakedGeometry (4), BlockStateDefinitions
(1), ModelBaker$Interner (1, **removed in 1.21.11**), ModelDiscovery
(1), ModelDiscovery$Slot (1), ModelManager$ResolvedModels (1): 16
classes, 117 methods, 0 named. The 26.x model-system refactor vs
1.21.11's older layout: 26.1 has `BlockStateModelSet` /
`BlockModelSet` / `FluidStateModelSet` + `LoadedBlockModels` +
`BuiltInBlockModels`, `ModelBaker.materials()` (MaterialBaker) +
`interner()`, `MissingCuboidModel`, and the `geometry/` / `cuboid/` /
`sprite/` packages; 1.21.11 has `BlockModelShaper` +
`SpecialBlockModelRenderer`, `ModelBaker.sprites()` (SpriteGetter) +
`parts()` (PartCache), `MissingBlockModel`, `BlockStateModel`, and
`TextureAtlasSprite` particle sprites. Material / ModelState /
SpriteGetter / UnbakedGeometry are "missing-side" (absent in 26.1 at
those paths — moved in 26.1).

**The real divergence — unbound states rendered nothing instead of
the missing cube (fixed, catalog.rs +96/-5).** Vanilla's
`ModelManager.createBlockStateToModelDispatch` fills EVERY state
with the missing model. The Rust's `StateModels` left `states[id]`
as None when a block's definition failed to load or didn't cover a
state; `each_seeded` errored ("unbound gallery block state") and
the mesher skipped the block, and `particle()` returned None. Only
observable with broken/incomplete resource packs (vanilla data
covers every state), but the missing-cube fallback is deliberate
vanilla behaviour. `StateModels` now bakes the missing cube once in
`from_facts_with_models` (kept out of `baked`, so face overrides
never touch it — vanilla's missing model isn't pack-overridable
either) and `each_seeded` / `particle()` fall back to it for
unbound states; a bake failure leaves the fallback out and unbound
states stay unbound, as before. `StateModels::default()`
(dedicated server) keeps missing: None — no behaviour change there.
New test `a_state_no_definition_covers_draws_the_missing_cube`: a
two-state block whose definition covers only one state — the
covered state draws its 1-face model, the uncovered state draws the
6-face missing cube (variant `minecraft:builtin/missing`) and its
particle is the cube's particle.

**Citation fixes (26.1 name → 1.21.11 name, jar-verified):**
1.21.11's `BlockStateModel` has `particleIcon()`
(TextureAtlasSprite); 26.1's has `particleMaterial()` (Baked) +
`materialFlags()`. `particleIcon` appears in 12 classes in the
1.21.11 client jar vs 1 in 26.1 (Minecraft only). Fixed three
comments: `blockstate/choice.rs:121`,
`models/catalog.rs:523`, `item_render.rs:197` — all →
`particleIcon`. No other 26.x citations found in the model files.

Verified, no change: `SpriteGetter.get` vs `MaterialBaker.get` —
1.21.11 is atlas-directed (ITEMS → item atlas, BLOCKS → block
atlas, `BLOCK_OR_ITEM` → item then block) while 26.1 always tries
item then block, but 1.21.11's `TextureSlots.parseTextureMap`
builds EVERY model texture slot as `BLOCK_OR_ITEM`, so all model
textures use item-then-block in both versions — and the Rust's
`Sprites` is a single texture-array namespace with the same
magenta/black missing fallback, so lookup order and the missing
fallback are both moot. `TextureSlots.Resolver.resolve` matches
the Rust's resolve.rs exactly (child-first entries, root-first
walk with child winning, fixpoint reference resolution, cyclic /
dangling aliases dropped). Parent-chain inheritance exact (AO
first-Some default true, gui light first-Some default Side,
geometry first-with-elements, transforms per-context first
non-Inherit, cycle → error). The missing model is the same 16³
cube in both versions (6 faces, UVs (0,0,16,16), tint −1, cull =
own direction, slot "missingno", particle → #missingno); only the
atlas differs, moot in the single-namespace sprite system.
`createBlockStateToModelDispatch` / `buildModelGroups` /
`requiresRender` logic identical; fluids already render through
the Rust's dedicated fluid renderer — the 1.21.11 shape
(LiquidBlockRenderer with sprites), not 26.1's
FluidStateModelSet. `ModelBaker.interner()` → `parts()` is internal
caching. `ParsedModels::load`, `block_library_or_missing` (failed
parse → skipped, resolve falls back to missing with a "Missing
block model" warn), built-ins winning over same-named resources,
and `item_generator_model()` all match vanilla. ResolvedModel
findTop* methods: identical logic, renames only.

Recorded, not changed: **`force_translucent` is a 26.x-ism**
(26.1's `Material` is `(sprite, forceTranslucent)` with an
object-form codec; 1.21.11's is `(atlasLocation, texture)` and
`parseTextureMap` calls `getAsString()` on every value, so an
object throws). Data-verified: 0 uses in 1.21.11 vanilla model
JSONs, 163 in 26.1's. Removing it would require rewriting the
model_resolution / model_baking oracles (still 26.x-shaped —
`VanillaModelResolutionOracle.java` imports
`net.minecraft.client.resources.model.sprite.Material` and cannot
compile against 1.21.11), the harness corpus, and 8+ Rust consumer
sites — T4 oracle-repair scope, explicitly deferred; unobservable
in vanilla 1.21.11 data. The Rust has no `requiresRender` /
model-groups mirror (remesh decision is performance-only, not a
rendered-result difference). The item-51 data-file migration gap
(save/world-data scope, version-neutral) is recorded there.

`cargo check -p horizon-assets -p horizon-client-render -p
horizon-client --lib --tests --no-default-features` clean (JOBS=1).
`cargo test -p horizon-client-render --lib models::catalog::` — 5
passed, 0 failed, including the new test. catalog.rs declares a
`#[path]` file mod (bounds_tests), so no rustfmt run — edits
hand-formatted.
Rank 2 now 231/492 (63 changed, 168 no-change).
work-items.json rank2.closed = 231 on 2026-11-20.

## Item 53 — crates/horizon-client/src/game/boats.rs (rank 2, CHANGED)

AbstractBoatRenderer (8, named submit), AbstractBoat (6, named
getGroundFriction, interact, modifyPassengerFluidInteractionBox,
positionRider, tick), ModelLayers (3), BoatRenderer (3, named
submitTypeAdditions), RaftRenderer (2), AbstractChestBoat (2,
named interact), ContainerEntity (1, named chestVehicleDestroyed):
7 classes, 25 methods, 9 named.

**The change — `WaterlilyBlock`, not `LilyPadBlock` (26.1 renamed
it).** The 1.21.11 mappings name it `WaterlilyBlock` (2
occurrences, 0 for `LilyPadBlock`), and the 1.21.11 prepared
`block-behaviours.json` gives the lily pad the class chain
`["WaterlilyBlock", "VegetationBlock", "Block"]`. The Rust matched
the 26.1 name in three places, so in the 1.21.11 build lily pads
counted toward `getGroundFriction` instead of being skipped
(boats.rs:651), boats never broke lily pads they ran into and the
lily-pad survival rule never applied (`classes.rs:191` —
`from_java` returned None for the 1.21.11 chain, so
`hooks.rs:583` `boat_inside` and the `survive.rs:395` rule never
fired), and the pad bound no survival rule at prepare time
(`survival.rs:72` — the 1.21.11 placement class is
`WaterlilyBlock`). All three fixed to the 1.21.11 name, plus four
comment citations (block_ticks.rs:134, game.rs:939, hooks.rs:578,
level/mod.rs:1299). Two pinning tests (machine-consumed
contracts): `Class::from_java("WaterlilyBlock") ==
Some(Class::LilyPad)` and `bind_definitions` binds the `LilyPad`
rule from the 1.21.11 placement class.

**Systematic finding (recorded, NOT fixed here — their items'
scope):** 14 of the 270 `classes.rs` literals appear nowhere in
the 1.21.11 prepared class chains. Verified rename map:
`FarmlandBlock` → `FarmBlock`, `PathBlock` → `DirtPathBlock`,
`RedstoneWireBlock` → `RedStoneWireBlock` (capital S),
`SnowyBlock`/`SpreadingSnowyBlock` →
`SnowyDirtBlock`/`SpreadingSnowyDirtBlock`,
`NetherFungusBlock` → `FungusBlock`, `NetherRootsBlock` →
`RootsBlock`; `AbstractBedBlock` and `SpeleothemBlock` have no
1.21.11 counterpart (BedBlock extends HorizontalDirectionalBlock
directly; PointedDripstoneBlock extends Block directly);
`PotentSulfurBlock`, `SulfurSpikeBlock`, `ShelfMushroomBlock`,
`StrawBedBlock` are 26.x-new blocks absent in 1.21.11. The same
renames sit in `survival.rs` CLASS_RULES (`PathBlock`,
`FarmlandBlock`, `SpeleothemBlock`, `ShelfMushroomBlock` —
farmland owner is `FarmBlock`, dirt-path owner `DirtPathBlock`,
dripstone owner `PointedDripstoneBlock` itself). Each rename needs
its own item's verification of the attached behaviour before the
literal changes — renaming blind could activate a 26.x-shaped
override — so this item fixes only WaterlilyBlock (fully
verified here). Full use-site table in rank2-item-53-TRIAGE.md.

**`modifyPassengerFluidInteractionBox` — 26.1-only; 1.21.11
short-circuits instead (verified, no Rust change).** 26.1's
`Entity.getFluidInteractionBox()` deflates the box 0.001 and lets
the vehicle clip it (AbstractBoat clips to above the hull, or
null when the hull covers it). 1.21.11 has no
`getFluidInteractionBox` at all: the water path short-circuits
for a passenger of a non-underwater boat (`wasTouchingWater =
false`, no water height / splash / fall reset / current push),
the lava path runs UNCLIPPED, and `updateFluidOnEyes` skips the
eye check when a non-underwater boat's box spans the eye height.
The Rust's player-path clip (`motion/fluids.rs`
`interaction_box()`, item 26) is equivalent for water + eyes
(boat not underwater ⟹ no water above the boat top, so the
clipped box holds no water — the same answer as the short-circuit).
Two gaps remain, both fluid-system scope (the shared
`EntityFluidInteraction` computes one box for all fluids;
per-fluid boxes would restructure it and the fluid_box harness):
(1) the player lava path is clipped where 1.21.11 runs it
unclipped — transient (the boat sinks and burns within ticks),
but real; (2) the mob path (`entities/fluids.rs`,
`game/entity.rs` `push_by_fluid`) has no boat short-circuit at
all — a mob riding a boat on water reads water immersion + push
where 1.21.11 reads none (observable mainly via
fire-extinguishing / `isInWater` logic).

Verified, no change: tick / floatBoat / controlBoat / paddles
(exact mirrors — buoyancy per status, invFriction 0.9/0.9/0.45/
0.9/land halved for a player controller, `(vy + buoyancy *
(0.04/0.65)) * 0.75`, paddle state `[right && !left || up, left
&& !right || up]`, paddle sounds at the π/4 stroke crossing with
`getPaddleSound`'s water/land/null by status, pitch 0.8+0.4·rand);
seat / positionRider (single 0.0 / chest 0.15 — javap-verified
both versions; two passengers 0.2/−0.6 + 0.2 for animals; ride
heights height/3 and height·0.8888889, javap-verified both
versions); pickup checks (area inflate (0.2, −0.01, 0.2), no
player steering, count < max, width < boat width,
`cannot_be_pushed_onto_boats` on the TYPE — the 1.21.11
`getType().is(tag)` form); boat_status; getGroundFriction apart
from the class name; interact (leash first, chest on sneak/full
seat, board otherwise, `out_of_control >= 60` gate — the 26.1
`Vec3 location` parameter is gone in 1.21.11, signature only,
and the Rust's boat interact consumes no hit location); chest
boat destroy/remove double dropContents, boat item only on the
death path with `entity_drops`, piglin anger on a player's
destroying hit with `entity_drops` (not creative), CONTAINER_OPEN
+ piglin anger on opening. `chestVehicleDestroyed`: 26.1
`instanceof Player` vs 1.21.11 `getType() == EntityType.PLAYER`
— equivalent. `Entity.is(tag)` vs `getType().is(tag)` (tick
pickup, positionRider): equivalent — 26.1's `is(TagKey)` comes
via `TypedInstance`/`Holder` and checks the type's tags; 1.21.11
has no `Entity.is(TagKey)`. Renderers: submit logic identical
(translate (0, 0.375, 0), YP (180−yRot), hurt tilt, bubble
wobble about (1,0,1), scale (−1,−1,1), YP 90°, water patch when
not underwater, rafts none) — the 26.1→1.21.11 change is plumbing
only (submitModel takes a texture Identifier in 26.1, a RenderType
via abstract `renderType()` in 1.21.11). ModelLayers: no
boat-layer changes; the 15 new baby saddle/armor layers and the
hanging-sign AttachmentType change belong to the mounts/signs
items. Report gaps: Boat, Raft, ChestBoat, ChestRaft, BoatModel,
RaftModel, VehicleEntity are in NO report item (their key methods
were verified identical anyway).

Pre-existing version-neutral approximations (recorded, not fixed):
`water_level_above` uses `ceil(maxY)` where vanilla uses
`ceil(maxY − lastYd)`; `boat_dismount_target` simplifies vanilla's
`getDismountLocationForPassenger`. Same code in both versions.

`cargo check -p horizon-world -p horizon-prepare -p horizon-client
--lib --tests --no-default-features` and `cargo check -p
horizon-client --bin horizon-client --tests --no-default-features`
both clean (JOBS=1). New tests pass (1 + 1). Single-token edits;
no rustfmt run.
Rank 2 now 232/492 (64 changed, 168 no-change).
work-items.json rank2.closed = 232 on 2026-11-20.

## Item 54 — crates/horizon-client-render/src/ui.rs (rank 2, CHANGED)

EditBox (11, named extractWidgetRenderState, insertText, setEditable,
setFocused, setValue), AbstractWidget (8, named
extractScrollingStringOverContents, handleCursor), AbstractSliderButton (3,
named extractWidgetRenderState), LockIconButton (2),
SpriteIconButton$TextAndIcon (2), TooltipRenderUtil (2), TextCursorUtils (1,
**removed in 1.21.11** — 26.1-only), LockIconButton$Icon (0): 8 classes,
29 methods, 8 named.

**The change — cursor moves fire the responder in 1.21.11.** 26.1's
`EditBox.moveCursorTo` ends with `updateTextPosition()`; 1.21.11's ends
with `onValueChange(value)` (javap-verified in both jars), so every
arrow-key / click / drag / Home / End / select-word cursor move calls the
responder in 1.21.11 and does not in 26.1. The Rust's chat screen mirrors
the responder as `ChatScreen::edited()` (`ChatScreen.onEdited`:
`allow_suggestions = true` + `update_command_info` + `isDraft = false`),
but `TextField::key` returned `Some(false)` for cursor keys,
`ChatScreen::click` never ran `edited()` after a click, and
`ChatScreen::shortcut` ran it only when the value changed — so command
suggestions did not refresh when the cursor moved in the chat box, and a
restored draft stayed a draft (grey italic, first Backspace clears all)
through arrow keys and clicks. Fixed: cursor keys now return `Some(true)`
(the bool means "the responder runs"); `click` calls `edited()` after
`input.click`; `shortcut` runs `edited()` for SelectAll (always fires via
`moveCursorTo`) and for Paste/Cut whenever `insertText` has room (fires
even when the value does not change) — Copy never fires, and a paste/cut
into a full box does not fire (`insertText`'s room check). The room is
computed before the shortcut spends it. Backspace/Delete still report the
deletion result: `deleteCharsToPos` fires only when `start != end`
(decompile-verified), so the existing `false` at the boundary and on an
empty value is already 1.21.11-exact. `insert_text` already returned
room>0 semantics. `edit_input` (the native/mod-API path) already fired
`edited()` on value/cursor/highlight change. New test
`chat_tests.rs::a_cursor_move_in_the_chat_box_runs_on_edited`: a restored
draft ends on an arrow key (Backspace then eats one character),
on a click, and on Select All; and the pending suggestions follow the
cursor ("hi St" offers only Steve, two arrow keys back to the empty word
offers Alex too).

**`EditBox.filter` is 1.21.11-only but dead API in vanilla:** no vanilla
class calls `EditBox.setFilter` (constant-pool scan of the 1.21.11 client
jar; the two byte-search hits were false positives —
`RecipeBookComponent.setFiltering` and
`SocialInteractionsPlayerList.setFilter`, confirmed by decompiling both),
and the default `nonNull` filter always passes. Unobservable; no Rust
change. The 1.21.11 `setValue` double-fire (via `moveCursorTo` plus
explicitly) is idempotent for every vanilla responder. IME preedit
(`preeditOverlay`, `preeditUpdated`, `onTextInputFocusChange`) is 26.1-only;
the Rust has no IME handling anywhere, so there is nothing to remove.
`TextCursorUtils` is a pure extraction: the blink formula, the insert
fill, and the "_" append cursor are identical in 1.21.11's inline form
(constants `CURSOR_INSERT_WIDTH` = 1, `CURSOR_APPEND_CHARACTER` = "_",
`CURSOR_BLINK_INTERVAL_MS` = 300, javap-verified in 1.21.11's `EditBox`).

**Citation fixes (26.1 name → 1.21.11 name, jar-verified):**
`GuiGraphicsExtractor.blitNineSlicedSprite/blitSprite/blit/requestCursor`
→ `GuiGraphics.…` (ui.rs, widgets.rs);
`AbstractWidget.extractScrollingStringOverContents` →
`renderScrollingStringOverContents` (ui.rs, graphics_style.rs,
screen_draw.rs, title.rs);
`AbstractSliderButton.extractWidgetRenderState` / `EditBox.extractWidgetRenderState`
→ `renderWidget` (ui.rs, screen_draw.rs, recipe_book_screen.rs);
`TextCursorUtils.isCursorVisible` → the inline 1.21.11 blink (ui.rs,
recipe_book_screen.rs). NOT fixed here (other items' scope):
`AbstractButton.extractDefaultLabel` / `extractDefaultSprite` (item 203);
`MultiLineEditBox.extractContents` / `extractWidgetRenderState` (item 529);
`AbstractSelectionList.extractWidgetRenderState` (items 13/87).

**Verified, no change:** `ui.rs` slider (track + 8px handle at
`value * (w-8)`, sprite rules, `slider_value` = `setValueFromMouse` +
clamp); `ui.rs` EditBox (insert/delete/word/click/drag/key/shortcut/draw,
blink formula — the 1.21.11 inline form); `TextField.scroll_to` (exact,
including the `pos == display` full-screen scroll-back);
`widgets.rs::widget_cursor` (base `handleCursor`, identical both
versions); `tooltip_box` ((x−12, y−12, w+24, h+24) = vanilla's
(x−3−9, y−3−9, w+3+3+18, h+3+3+18), first line 12, rest 10);
`title.rs` TextAndIcon and `screens.rs` CenteredIcon (exact geometry);
`append_field` blink + cursors. The 1.21.11 slider cursor difference
(inactive hovered slider → POINTING_HAND, not NOT_ALLOWED) is
unobservable: the Rust implements no slider cursor at all (no ResizeEw /
IBeam variants, no call sites) — a pre-existing, version-neutral gap, as
are the missing LockIconButton mirror, the missing `ui::EditBox` hint,
`ui::EditBox::scroll_to` lacking the `pos == display` branch,
`ui::EditBox::set_cursor` resetting the blink, `TextField::click` lacking
double-click word selection, the chat input having no drag handler, and
max length counting chars (Rust) vs UTF-16 units (vanilla).

Verification: `cargo check -p horizon-client-render -p horizon-client
--lib --tests --no-default-features` clean (JOBS=1, 2m25s). `cargo test -p
horizon-client-render --lib chat::tests::` — 27 passed, 0 failed, including
the new test. No rustfmt run (comment-only edits plus small hand-formatted
code edits).
Rank 2 now 233/492 (65 changed, 168 no-change).
work-items.json rank2.closed = 233 on 2026-11-20.

## Item 55 — crates/horizon-client/src/advancements.rs (rank 2, CHANGED)

KilledTrigger$TriggerInstance (10), PlayerPredicate (8, named matches),
EntityHasScoreCondition (8, named test), LootItemRandomChanceCondition (4,
named test), WeatherCheck (4, named test), KilledByArrowTrigger$TriggerInstance
(3), PlayerPredicate$Builder (3), DataComponentExactPredicate (3, named test),
PlayerPredicate$AdvancementCriterionsPredicate (2),
PlayerPredicate$AdvancementDonePredicate (2), AdvancementProgress (1),
CollectionPredicate (1), ContextAwarePredicate (1), FoodPredicate (1,
**removed in 1.21.11**), FoodPredicate$Builder (1, **removed in 1.21.11**),
RecipeCraftedTrigger$TriggerInstance (1), Validatable (1, **removed in
1.21.11**), WeatherCheck$Builder (1): 18 classes, 55 methods, 5 named.

**The change — `PlayerPredicate` has no food in 1.21.11.** 26.1's
`PlayerPredicate` is an 8-field record (level, food, gamemode, stats,
recipes, advancements, lookingAt, input); 1.21.11's is 7 fields — the
`food` field (`FoodPredicate`) is gone, the class does not exist in the
1.21.11 jar, the CODEC drops the `"food"` optional field (P8 → P7), and
`Builder` loses `setFood`. The Rust mirrored the 26.1 food check in
`player_facts` (food level in `level`, saturation widened to double in
`saturation`, against `PlayerFacts.food`, always filled from
`Player.getFoodData` for the local player and LAN guests). Removed: the
`player_facts` food block, the `PlayerFacts.food` field + Debug entry,
and both fill sites in game/advancements.rs (the local `player_facts`
guard simplified to `is_none()` since `p` is no longer read). A `food`
key in a predicate JSON is now ignored, as the codec ignores unknown
fields. The test pinning the 26.1 behaviour
(`player_food_tests_level_and_saturation_in_their_bounds`, including the
1.2f→double widening edge) is replaced by
`a_food_field_in_a_player_predicate_is_ignored`. `FoodData` itself (the
hunger mechanic) is untouched — only the predicate mirror is removed.
`FactsAsked` never indexed food, so no index change.

**Citation fix:** `predicate_references` cited
"`Validatable.validateHolder`'s active-name rule" — `Validatable` is
removed in 1.21.11, and no `validateHolder` exists on 26.1's
`Validatable` either (javap-verified). The visited-set cycle rule lives
in 1.21.11's `ValidationContext` (`enterElement` / `hasVisitedElement`,
decompile-verified); the citation now says so.

**Verified, no change:** `killed` (KilledTrigger — the `entity` →
`entityPredicate` rename is Java-internal; the codec field stays
`"entity"`); `killed_by_arrow` (weapon, distinct-victim matching,
unique-entity-type count); `literal_random_chance` (`draw() < chance`,
always draws); entity_scores (missing objective/score fails, constant
integer ranges); weather_check (absent fields impose no constraint);
`components` (DataComponentExactPredicate.test — every expected component
equals what the entity reports); `potions` (CollectionPredicate —
contains + count + size); `Progress` (done/grant/revoke, `from_json`
ignoring "done" as vanilla's CODEC does, `to_json` writing derived done,
`counts()` = vanilla's percent/progress-text inputs); the player
predicate arm (instanceof player → gamemode array membership —
`GameTypePredicate.CODEC` is a list codec in BOTH versions, javap-verified
→ lookingAt → input → `player_facts`). The loot conditions' `MAP_CODEC`
→ `CODEC` / `codec()` → `getType()` renames and the
`getReferencedContextParams`/`validate` plumbing changes touch no
`test()` body. `AdvancementProgress`, `CollectionPredicate`,
`ContextAwarePredicate.matches`, `AdvancementCriterionsPredicate` /
`AdvancementDonePredicate` / `StatMatcher` are identical apart from
visibility/generics.

**Recorded, not changed:** a single-string `"gamemode"` fails to decode
in vanilla (list codec) but passes unconstrained in the Rust
(version-neutral; vanilla data always uses lists);
`literal_random_chance` supports only literal float chances and
entity_scores only `entity == "this"` with constant integer ranges
(documented bounded contexts); the `input` mirror covers sneak only.

Verification: `cargo check -p horizon-client --lib --tests
--no-default-features` clean (JOBS=1, 3m35s). `cargo test -p
horizon-client-lib --lib advancements::` — 55 passed, 0 failed, 1
ignored, including the new test and all 5 player predicate tests.
No rustfmt run (advancements.rs declares test modules; game/advancements.rs
is a file mod).
Rank 2 now 234/492 (66 changed, 168 no-change).
work-items.json rank2.closed = 234 on 2026-11-20.

## Item 56 — crates/horizon-client/src/item_render/special.rs (rank 2, CHANGED)

BannerRenderer (25, named getExtents/submit/submitPatterns),
ShieldSpecialRenderer (9, named submit), PlayerHeadSpecialRenderer (7),
PlayerHeadSpecialRenderer$Unbaked (2), ShieldSpecialRenderer$Unbaked (2),
SpecialModelRenderer (2, named submit),
SpecialModelRenderer$BakingContext (2), EndCubeSpecialRenderer (1,
**removed in 1.21.11**), EndCubeSpecialRenderer$Type (1, **removed in
1.21.11**), EndCubeSpecialRenderer$Unbaked (1, **removed in 1.21.11**): 10
classes, 52 methods, 5 named.

**The change — the per-model transform moved from the item model into
`submit`/`getExtents`.** In 26.1 every special item model carries a
`transformation` in its JSON (shield scale (1,−1,−1); banner translate
(0.5,0,0.5) + scale (2/3,−2/3,−2/3); skulls rotX-180° + translate
(0.5,0,0.5); shulker rotX-180° + scale 0.9995 + translate (0.5,1.4995,0.5);
conduit translate (0.5,0.5,0.5); statue translate (0.5,1.5,0.5) + scale
(1,−1,−1); trident/shield also had a `DEFAULT_TRANSFORMATION`), and the
renderers submitted untransformed. In 1.21.11 the item models have NO
`transformation` field (verified from the 1.21.11 jar's shield/banner/
player_head/skeleton_skull/shulker_box/conduit/statue JSONs); each
renderer applies its transform inline in `submit` AND `getExtents`. The
Rust was written against the 26.x shape (`layer.transform` is identity
for 1.21.11 data), so `special::submit` drew every special item
untransformed — banners full-size at the origin, shields/tridents
upside-down, heads unflipped and uncentred, shulkers/conduits/statues
unplaced. Fixed with a new `item_transform(&SpecialKind)` helper applied
in `submit` (item path only) and `extents`: banner T(0.5,0,0.5) ·
S(2/3,−2/3,−2/3) at angle 0; shield/trident S(1,−1,−1); head/player-head
T(0.5,0,0.5) · R_y(180°) · S(−1,−1,1) (the pose-level form of
`submitSkull(null, 180, 0)`'s translate + scale + state yRot — the same
algebra the item-16 block-entity skull path uses); conduit
T(0.5,0.5,0.5); shulker `prepareModel` (T(0.5,0.5,0.5) · S(0.9995) ·
R(orientation) · S(1,−1,−1) · T(0,−1,0), verified equal to the 26.1 JSON
transform for UP); statue `positionModel` (T(0.5,1.5,0.5) · S(−1,−1,1)).
Chest and decorated pot have no transform in either version.
`statue_pose` also gains the 1.21.11 `setupAnim(SOUTH)` root yRot (π;
26.x's `setupAnim(Unit)` set only y = 0, zRot = π).

**The worn-skull path must not take the item transform.**
`ItemRenderer::submit_special` (armor stands, mob heads) passes `head ·
scale(1.1875)`. Vanilla 1.21.11 `CustomHeadLayer` composes
scale(1.1875,−1.1875,−1.1875) · translate(−0.5,0,−0.5) ·
submitSkull(null, 180, 0): the translates cancel and the flips plus the
180° yaw compose to a uniform 1.1875 scale, so the Rust's worn pose is
already exactly right. `submit()` (item path) applies the transforms;
new `submit_worn()` does not; `submit_special` calls the latter.
(CustomHeadLayer itself is item 391.)

**Codec changes (Unbaked codecs, decompile-verified):** 1.21.11 drops the
banner `attachment` field (26.1: optional, default GROUND) and the chest
`chest_type` field (26.1: optional, default SINGLE; bake always uses
`ModelLayers.CHEST`) — `wall` removed from `SpecialKind::Banner`
(banner items are always standing) and `chest_type` from
`SpecialKind::Chest` (always the single chest model; `chest_model`
deleted). 1.21.11 ADDS the shulker `orientation` field (optional,
default UP) — added to `SpecialKind::ShulkerBox`, applied via
`Direction.getRotation` (mirrors `block_entity_render::direction_rotation`,
item 16). Skull codecs are identical.

**Report gaps (changed classes in NO report item — fixed here, recorded
loudly):** `BannerSpecialRenderer`, `SkullSpecialRenderer`,
`ShulkerBoxSpecialRenderer`, `TridentSpecialRenderer`,
`ConduitSpecialRenderer`, `ChestSpecialRenderer`,
`DecoratedPotSpecialRenderer`, `CopperGolemStatueSpecialRenderer` all
differ between the jars (javap md5s) but appear in no item; the
transform/codec work above fixes the first six plus the statue, and
`DecoratedPotSpecialRenderer` delegates to `DecoratedPotRenderer` (item
185) with no transform of its own. `BedSpecialRenderer`,
`StandingSignSpecialRenderer`, `HangingSignSpecialRenderer` are also
changed, also in no item, and the Rust has NO `SpecialKind` for them:
bed/sign items fail `SpecialModel::decode` and fall back to the missing
model. NOT fixed here (new kinds + renderers); recorded as a gap for the
sweep.

**Verified, no change:** the block-entity banner path (`banner_parts` +
`render_banner` — translate + rotY(−angle) + scale matches 1.21.11's
`submitBanner` exactly, phase formula identical, layer order pole/flag/
base colour/≤16 matches, textures `entity/banner_base` +
`entity/banner/base` verified against `ModelBakery.BANNER_BASE` /
`Sheets.BANNER_BASE`); the shield handle/plate split (1.21.11 draws the
handle separately, then the whole model in `submitPatterns` — same
pixels as the Rust's whole-model-per-layer draws); the shield glint
layering (1.21.11 submits the glint between the untinted base and the
colour layers, but `entityGlint` is a translucent pass drawn after the
cutout pattern layers, so the visible result is still glint-on-top —
the Rust's per-layer foil and the existing test stand); the pattern-layer
render type (26.1 `RenderTypes.bannerPattern`, TRANSLUCENT blend, no
depth write → 1.21.11 `entityNoOutline`, cutout — identical pixels for
0/255-alpha pattern textures; the Rust's plain quads match the 1.21.11
cutout form); the banner phase formula, 16-layer cap, shield
base-colour fallback (WHITE), and pattern texture ids (already 1.21.11
from item 46); the player-head render type (info's, else
`DEFAULT_PLAYER_SKIN_RENDER_TYPE`, the Rust's `OitEntity` sink) and bake
(same `PLAYER_HEAD` model, nullable in 1.21.11); the
`SpecialModelRenderer.submit` `ItemDisplayContext` parameter (ignored by
all three renderers) and `BakingContext.sprites()` → `materials()`
(Java plumbing); `submit_banner_result` (loom preview — its pose comes
from `banner_result_pose`; `GuiBannerResultRenderer` is item 506, rank
3); the decorated-pot item submit (no transform in either version;
extents cover neck/top/bottom only); EndCube (26.1-only, renders the
end-portal/gateway cube; the Rust has no end_cube kind and decode
rejects it, as 1.21.11 vanilla does); culling (the Rust's
`cull_back = false` matches 1.21.11's NoCull types where they matter —
skull `entityCutoutNoCullZOffset`, statue `entityCutoutNoCull` — and is
unobservable for closed geometry elsewhere; the pot already culls
explicitly).

Verification: `cargo check -p horizon-assets -p horizon-client --lib
--tests --no-default-features` clean (JOBS=1, 2m11s; re-run clean after
rustfmt, 4m19s). `cargo test -p
horizon-assets --lib item_model::special::` — 2 passed (new codec
tests, re-run after rustfmt). `cargo test -p horizon-client-lib --lib
item_render::special::` — 7 passed, 0 failed, including the 2 new tests
(`item_transforms_match_the_1_21_11_inline_forms`,
`submit_applies_the_item_transform_and_submit_worn_does_not`) and the
existing shield-glint and pot tests (8m25s at JOBS=1; the
horizon-client lib test link was OOM-killed, the known fallback path).
rustfmt --edition 2024 applied to the two edited leaf files (assets
special.rs, item_render special.rs; the latter also picks up 2
pre-existing formatting spots in `submit_book` and the banner arm).
Rank 2 now 235/492 (67 changed, 168 no-change).
work-items.json rank2.closed = 235 on 2026-11-20.

## Item 57 — crates/horizon-client-render/src/mesher.rs (rank 2, CHANGED)

ModelBlockRenderer (25, named tesselateBlock/tesselateFlat), BlockColors
(13, named createDefault), BiomeColors (10), MovingBlockRenderState (4),
BlockTintCache (2), BlockTintCache$CacheData (1), BlockTintSource (1,
**removed in 1.21.11**), BlockModelLighter (1, **removed in 1.21.11**) +
$1, $AdjacencyInfo, $AmbientVertexRemap, $Cache, $Cache$1, $Cache$2,
$SizeInfo (all **removed in 1.21.11**), FluidRenderer (1, **removed in
1.21.11**) + $1, $Output (both **removed in 1.21.11**), CardinalLighting
(1, **removed in 1.21.11**) + $1, $Type (both **removed in 1.21.11**),
HalfTransparentBlock (1): 22 classes, 71 methods, 3 named.

**The change — two 1.21.11 deltas in the mesher's light/colour math.**
First, the AO light blend: 26.1's `LightCoordsUtil.smoothBlend` borrows
the centre's sky AND block channels per sample when the centre is bright
(sky > 2 or block > 2); 1.21.11's `ModelBlockRenderer.blend` (private
static) has no gate and no borrowing — a sample that is exactly 0 takes
the centre, then the same `(a+b+c+center)>>2 & 0xFF00FF` average. The
Rust's `smooth_blend` was a hybrid of the two (gate + zero→centre +
sky-borrow only) that matched neither version; it is rewritten to
1.21.11's form and renamed `blend`. The weighted blend is identical in
both versions (26.1 `smoothWeightedBlend` == 1.21.11's 8-arg `blend` ==
the Rust's `weighted_blend`). Second, the vertex colour arithmetic: 26.1
floors the brightness to a gray level, scales it by the cardinal shade,
then multiplies by the tint in integers (`ARGB.gray`, `scaleRGB`,
`multiply` — the Rust mirrored this exactly); 1.21.11 passes the tint as
floats (`ARGB.redFloat` etc.) and `VertexConsumer.putBulkData` computes
`ARGB.colorFromFloat(1.0, brightness*r, brightness*g, brightness*b)`
with `as8BitChannel` = `Mth.floor(v * 255)` — one floor at the end, no
intermediate gray level, no integer division — up to 1 off per channel
(AO 0.55 × shade 0.8, tint 200: 88 vs 87; flat shade 0.5, tint 100: 50
vs 49; both computed independently in f32). `Mesher::face` now carries
the per-vertex float brightness (AO × shade, or shade alone when flat)
and computes each channel with the new `lighting::lit_channel`, which
reuses the tested `gray()` for the `Mth.floor` step in Java's f32 op
order. The `separate_ao` (Iris) path is unchanged. Bundled
version-neutral correction in the same lines: an unshaded quad
(`shade == false`) took `directional = 1.0`, but both versions use the
up-face shade for it (`getShade(dir, false)` / `cardinalLighting.up()` =
0.9 in the Nether, 1.0 elsewhere); the new `lighting::face_shade`
returns the up-face value when the quad does not shade.

The rest of the item is a restructure with no behaviour change, all
verified from the jars. 26.1's `BlockModelLighter` is inlined into
1.21.11's `ModelBlockRenderer` (inner `AmbientOcclusionRenderStorage` /
`CommonRenderStorage` / `Cache` / `AdjacencyInfo` /
`AmbientVertexRemap` / `SizeInfo`); the AO is line-for-line the same —
face bounds, `facePartial`/`faceCubic`, 4 corner + 4 diagonal samples,
the translucent-corner rule, the centre-light rule, the non-cubic
weighted blend, the diagonal fallback to corner[0]'s shade/light when
both adjacent corners are closed, the AdjacencyInfo weight tables, the
AmbientVertexRemap values, the 100-entry caches — and the Rust's
`ambient_occlusion` mirrors all of it. 26.1's `CardinalLighting` record
becomes 1.21.11's `DimensionType.CardinalLightType` +
`ClientLevel.getShade` with the same values (verified from the 1.21.11
bytecode: shade=false 0.9 Nether / 1.0 else; DOWN 0.9/0.5, UP 0.9/1.0,
N/S 0.8, W/E 0.6). 26.1's `BlockTintSource` system becomes 1.21.11's
`BlockColor` lambdas with an identical registration set (same blocks,
same constants, same stem/redstone formulas, same tall-grass `below()`
rule, same pink-petals blank layer 0); the Rust's tint table matches
both. `BlockTintCache` / `BiomeColors` identical apart from renames;
`HalfTransparentBlock.skipRendering` identical; `MovingBlockRenderState`
drops `cardinalLighting` + `lightEngine` for a delegating `level`;
`LevelRenderer.getLightCoords` (26.1) == `getLightColor` (1.21.11);
`LightTexture.lightCoordsWithEmission` == 26.1's
`LightCoordsUtil.lightCoordsWithEmission`; `getShadeBrightness`
overrides (SoulSand/Mud 0.2, SnowLayer layers==8, TransparentBlock 1.0)
verified in the 1.21.11 jar; `emissiveRendering` has exactly two setters
in both jars (magma, always; sculk sensor, phase == ACTIVE; calibrated
inherits via `ofLegacyCopy`); `TerrainParticle`'s grass-block untint is
an explicit check in 1.21.11 vs `colorAsTerrainParticle` returning −1 in
26.1, same result; `Block.getOffset`, `forceOpaque` (a boolean in 26.1,
same decision in 1.21.11's render-type lookup), the tint-cache and
should-render-face caching all match.

Referrals: `LiquidBlockRenderer` → item 754 (rank 4; `mesher/fluid.rs`
untouched — the mesher.rs header now cites the 1.21.11 name);
`LightCoordsUtil`'s other functions → item 191; `BlockTintSources` →
item 107; `VertexConsumer.putBulkData`'s other consumers → item 326;
`BlockElement`'s 26.x-only `shade_direction_override` / `color` model
keys (1.21.11's `BlockElement` has only `shade` + `lightEmission`; the
Rust parses and applies them, dead for vanilla 1.21.11 models) → item
750; moving blocks' `baked_model_vertices` (fixed DEFAULT cardinal
shades, no Nether variant, integer tint multiply) → item 754;
`LeavesBlock` → 272, `SectionCompiler` → 175, `LevelRenderer` /
`BrightnessGetter` → 503 / 206. Known version-neutral gaps recorded,
not fixed: tall-grass upper halves sample the grass tint at their own
position instead of `pos.below()` (needs cross-layer tint state);
the AO corner-permeability proxy (`!isSolidRender || dampening == 0`
vs vanilla's `!isViewBlocking || light == 0`, default isViewBlocking =
`blocksMotion && isCollisionShapeFullBlock` — can differ for the
`isViewBlocking(never)` overrides with a full occlusion shape: soul
sand, mud, 8-layer snow, cherry/pale-oak leaves, pending a check
against the prepared facts); 26.3's `BlockState.isLightPermeable()` and
`BlockColors.getTintSources` (used by the 26.3 section-mesh oracle) do
not exist in 1.21.11 — the oracle needs its T4 port.

Verification: `cargo check -p horizon-client-render --lib --tests
--no-default-features` clean (JOBS=1, 3m19s). `cargo test -p
horizon-client-render --lib --no-default-features mesher::lighting::` —
11 passed, 0 failed, including the 3 new/updated tests
(`blend_replaces_zero_samples_with_the_centre`,
`lit_channel_floors_the_float_product_once`,
`face_shade_uses_the_up_face_without_shading`); `mesher::` as a whole —
67 passed, 0 failed, 24 ignored (fixture-needing, run on CI). rustfmt
--edition 2024 applied to the edited leaf file (mesher/lighting.rs);
mesher.rs is a module root and was hand-formatted to match.
Rank 2 now 236/492 (68 changed, 168 no-change).
work-items.json rank2.closed = 236 on 2026-11-20.

## Item 58 — crates/horizon-client/src/game/enchant_effects.rs (rank 2, NO-CHANGE)

EnchantmentHelper (10, named doPostPiercingAttackEffects, enchantItem,
getAvailableEnchantmentResults, getEnchantmentLevel,
getItemEnchantmentLevel), EnchantmentAttributeEffect (2),
PlaySoundEffect (2, named apply), SpawnParticlesEffect (2, named apply),
SpawnParticlesEffect$VelocitySource (2), ApplyEntityImpulse (1),
ReplaceDisk (1, named apply), UnobstructedPredicate (1): 8 classes, 21
methods, 8 named.

Every delta is a rename or plumbing, except two narrowings the Rust
already implements in its 1.21.11 shape. The lunge rename:
26.1 `LivingEntity.postPiercingAttack()` →
`EnchantmentHelper.doPostPiercingAttackEffects` →
`Enchantment.doPostPiercingAttack`; 1.21.11
`Player.lungeForwardMaybe()` (food gate) →
`LivingEntity.lungeForwardMaybe()` (server-side only) →
`EnchantmentHelper.doLungeEffects` (instanceof LivingEntity guard) →
`Enchantment.doLunge`; the component id `minecraft:post_piercing_attack`
is unchanged in both versions, and the Rust's `post_piercing_attack`
already cites `doLungeEffects`, reads that component, and is called from
spear.rs after the jab. The food term moved out of `lunge.json`'s
requirements (26.1's any_of: not-a-player / creative / food ≥ 7) into
`Player.lungeForwardMaybe`'s `hasEnoughFoodToDoExhaustiveManoeuvres()`
in 1.21.11; the Rust's gate (not riding / not fall-flying / not in
water, plus creative or food ≥ 7) is already the 1.21.11 shape. The
impulse grace narrowed from LivingEntity to Player
(`ApplyEntityImpulse.apply` calls `applyPostImpulseGraceTime(10)` in
both, same body); the Rust's `ImpulseContext` lives on the player only
(living_fall.rs already records the split) and the lunge path runs only
for the player. The impulse rotation matches: Java's
`Vec3.addLocalCoordinates` (dir.z·forward + dir.y·up + dir.x·right from
the look basis) expands to exactly the Rust's pitch-then-yaw formula.
Everything else in EnchantmentHelper is identical (all modify/process/
doPostAttack/location/tick/enchant-selection helpers, `getRandomItemWith`
in EquipmentSlot order from the entity random); `EnchantmentAttribute-
Effect` is a `MAP_CODEC` → `CODEC` rename (`idForSlot`'s `id/slot`
suffix matches the Rust's format); PlaySound/SpawnParticles are
`FloatProviders` → `FloatProvider` renames (the Rust's `soul_speed_tick`
matches the soul-speed data in both versions: particles first, then the
35% sound, same draw order); ReplaceDisk is a `getState` signature
change (the disk loop matches term by term); UnobstructedPredicate lost
`final` on its CODEC. No Rust edits.

Referrals: `Enchantment` → item 12 (closed); `Avatar` → item 312;
`Vec3.addLocalCoordinates` → item 332. Known version-neutral gaps
recorded, not fixed: the `apply_impulse` arm never calls
`apply_grace_time(10)`; `post_piercing_attack` hardcodes the vanilla
lunge gate and skips `matchingSlot` and the JSON requirements; the
`play_sound` arm ignores `isSilent()` and reads volume/pitch as plain
floats; `replace_disk` only fires the block_place game event; the
impulse arm uses libm trig where Java uses Mth's sine table.

Verification: `cargo check -p horizon-client --lib --tests
--no-default-features` clean (JOBS=1, 3m02s).
Rank 2 now 237/492 (68 changed, 169 no-change).
work-items.json rank2.closed = 237 on 2026-11-20.

## Item 59 — crates/horizon-client/src/animate_tick.rs (rank 2, CHANGED)

9 classes, 18 methods, 8 named: HangingMossBlock, CampfireBlockEntity,
LightningRodBlock, AmbientDesertBlockSoundsPlayer, DryVegetationBlock,
EyeblossomBlock, ShortDryGrassBlock, TallDryGrassBlock, EyeblossomBlock$Type.

Two fixes. (1) HangingMoss bonemeal target drops the build-height guard:
26.1 `isValidBonemealTarget` is `canGrowInto(tip.below()) &&
isInsideBuildHeight(tip.below())`; 1.21.11 is `canGrowInto` only — the
same 26.x guard pattern item 27 removed elsewhere (this discharges item
27's HangingMoss referral; Bamboo, item 95, is still open). The
`performBonemeal` arm and `hanging_moss_tip` (`getTip`) already
matched. New test `hanging_moss_bonemeals_at_the_bottom_of_the_world`
(growth_tests.rs): tip at min_y, meal used, nothing placed below.
(2) The dry-vegetation support tag is `dry_vegetation_may_place_on` in
1.21.11; 26.1 renamed it `supports_dry_vegetation` (and changed its
contents: 1.21.11 = #sand, #terracotta, #dirt, farmland; 26.1 = #sand,
#terracotta, #supports_vegetation). The 26.1 name was resolved in three
places, all fixed: `horizon-prepare/src/survival.rs` VEGETATION_RULES
(the prepare pipeline's `bind_definitions`; the exporter records the
placement class but the tag is a static constant, so the hardcoded
table is the only source), `tools/test_block_survival_parity.py`
VEGETATION_RULES (the harness driver, which itself produces
`survival-rules.json`), and the live survival path (`level/data.rs`
`Tags` table + the `Class::DryVegetation` arm in `level/survive.rs`).
`Tags::resolve` silently substitutes an EMPTY set for a missing tag, so
with the 26.1 name dry grass could never survive on any ground. New
test `dry_vegetation_binds_the_1_21_11_support_tag` pins the binding
(short_dry_grass: owner VegetationBlock, placement DryVegetationBlock;
both dry-grass blocks extend DryVegetationBlock and neither overrides
`mayPlaceOn`, verified from the 1.21.11 jar). The short/tall dry-grass
bonemeal targets were already 1.21.11-shaped (item 27): short is
always a target, tall is `hasSpreadableNeighbourPos` only.

Verified identical, no change: EyeblossomBlock.animateTick (nextInt
700, below pale_moss_block → EYEBLOSSOM_IDLE; `randomTick`/`tick`/
`tryChangingState` identical), HangingMossBlock.animateTick (nextInt
500, above pale-oak logs/leaves), LightningRodBlock.animateTick
(thundering, nextInt(200) <= gameTime%200, y == world surface − 1,
sparks along the facing axis, UniformInt 1..2; the Rust uses
`level_random()` here, matching Java), CampfireBlockEntity
(`cookTick`/`cooldownTick`/`particleTick`: 0.11 chance, 2+nextInt(2)
puffs, per-slot 0.2 chance, 4 smoke at the slot corner, velocity
(0, 5.0E-4, 0); the Rust's re-drawn loop bound and 2D-direction table
match), AmbientDesertBlockSoundsPlayer (sand 2100, dry grass 200,
dead bush 130 + nextInt(3) gate; `sand_sound_nearby` matches the
≥3-matches early-exit and the far-column scan exactly).

Referral — the `supports_*` tag family is 26.1-new: NONE of the
`supports_*` block tags exist in the 1.21.11 jar (verified:
supports_azalea/cactus/sugar_cane/bamboo/cocoa/lily_pad/frogspawn/
mangrove_propagule/nether_sprouts/nether_wart/wither_rose/crops/
big_dripleaf all absent). 1.21.11 uses differently-named tags
(`bamboo_plantable_on`, `azalea_grows_on`) or hardcoded checks. Every
Rust lookup of a `supports_*` tag silently resolves to the empty set,
so those blocks' survival is broken until their own items audit them.
The Python driver's VEGETATION_RULES still says `LilyPadBlock` where
the Rust table (item 53) says `WaterlilyBlock` — an item-53 loose end
in the driver. Known version-neutral gap recorded, not fixed: the
animate-tick arms draw from the particle-provider random stream where
Java draws everything from the level random (odds match, exact
sequences differ; affects every arm).

Verification: `cargo check -p horizon-world --lib --tests
--no-default-features` clean (JOBS=1, 1m50s); `cargo test
-p horizon-prepare --lib survival::` 2 passed; `cargo test
-p horizon-world --lib --no-default-features
level::growth_tests::hanging_moss_bonemeals_at_the_bottom_of_the_world`
1 passed (fixture-skipped locally, runs on CI); `python3 -m py_compile
tools/test_block_survival_parity.py` OK; rustfmt --check clean on all
five edited Rust files.
Rank 2 now 238/492 (69 changed, 169 no-change).
work-items.json rank2.closed = 238 on 2026-11-20.

## Item 60 — crates/horizon-client/src/entity_nbt.rs (rank 2, NO-CHANGE)

13 classes, 56 methods, 4 named. Javap diff +205/-317 (839 lines),
decompile diff 1612 lines, both read in full Nov 20, 2026. No Rust
edits. Triage note `rank2-item-60-TRIAGE.md` (workdir root, scratch).

The one real behaviour delta in these classes is the pig sound variant,
and it is 26.1-only: 26.1's `Pig` has `DATA_SOUND_VARIANT_ID`,
`saves/loads sound_variant`, routes sounds through `getSoundSet()`
(baby vs adult sets), overrides `playEatingSound`, handles the
`PIG_SOUND_VARIANT` component, and picks a random sound variant in
`finalizeSpawn`. 1.21.11 has none of it — fixed `PIG_AMBIENT/HURT/DEATH/
STEP` sounds, no `playEatingSound` override. The Rust is already
1.21.11-shaped throughout: `finalize::sound_variants` is non-empty
only for wolves ("only wolves have them in 1.21.11"); entity_nbt writes
`sound_variant` only for wolves and the test
`farm_animals_keep_their_variant_and_wolves_their_sound_variant` pins
that a pig never writes it; pig sounds are the fixed `entity.pig.*`
events; `eating_sound` has an explicit comment that pigs have no
eating sound in 1.21.11; `finalize` picks the variant alone for
pig/cow/chicken; no `pig_sound_variant` in the component registry.

Pig entity-data indices verified 1.21.11-correct by counting `defineId`
calls per class in both jars (Entity 8, LivingEntity 7, Mob 1,
AgeableMob 1 in 1.21.11 vs 2 in 26.1 — the age-lock item 23 removed):
baby 16, pig boost 17, pig variant 18. `remote_mob_data.rs` and
`lan.rs` both use 17/18. The 1.21.11 serializer remap was verified
against the true `registerSerializer` call order in both jars' static
initializers (field declaration order is NOT registration order):
1.21.11 has 39 serializers (0–38), 26.1 has 43; the Rust's `V1_21_11`
`up`/`down` runs map every id correctly, and 1.21.11 has no cat/cow/
pig/chicken sound-variant serializers (only wolf). No translation runs
for a 1.21.11 server anyway (`older_server` returns None for the
pinned version), so `remote_mob_data::apply` sees raw 1.21.11 wire
indices. Item 335's referral (verify the serializer remap) is answered
for the 1.21.11 table.

`ExpirableValue` is a Record with `Optional<Long>` ttl in 26.1 and a
plain mutable class with a `long` ttl (`Long.MAX_VALUE` = never
expires) in 1.21.11; the codec wire shape is identical. The Rust's
`Slot<T>` is already the 1.21.11 shape, and the expiry semantics were
verified against `Brain.forgetOutdatedMemories` bytecode in the
1.21.11 jar (erase-if-expired, then tick — a ttl-1 memory is visible
during the set tick plus one more, exactly the Rust's behaviour).

`ConversionType` is identical in both versions (`entityTags` →
`getTags` rename only); the Rust's `mobs::conversion::single` covers
the modelled fields, with command tags travelling via
`entity_nbt::conversion_tags`. `AngerManagement` is identical except
`highestAnger` visibility and the ctor's unseeded random source; the
Rust's warden mirror matches, including `sort_and_update_highest`.
`EvokerFangs` (+0/-0) and `TagValueInput` + wrappers are identical
apart from visibility and compiler artifacts; the Rust's NBT
(`Warmup`/`Owner`) and `game/fangs.rs` tick match.

Known version-neutral gaps, recorded not fixed: `conversion::single`
does not copy absorption, the ANGRY_AT brain memory, the on-fire flag,
the portal cooldown, the team, or CUSTOM_DATA, and it copies
`invulnerable_time` although neither version's `convertCommon` does
(its comment contradicts the code); warden anger starts
`conversion_delay` at 0 instead of drawing 0..=2 and does not transfer
anger to the UUID map on unload-removals; the breeding variant coin
for pigs/cows/chickens/foxes/cats/wolves is drawn from the level random
where Java draws `this.random.nextBoolean()` from the mother's random
(the axolotl/mooshroom arms use the mother's random correctly).

Verification, Nov 20, 2026: `cargo check -p horizon-client --lib
--tests --no-default-features` clean (JOBS=1).
Rank 2 now 239/492 (69 changed, 170 no-change).
work-items.json rank2.closed = 239 on 2026-11-20.

## Item 61 — crates/horizon-client/src/game/sign_edit.rs (rank 2, CHANGED)

5 classes, 26 methods, 7 named. Javap diff +448/-534 (1149 lines),
decompile diff 617 lines, both read in full Nov 20, 2026. Triage note
`rank2-item-61-TRIAGE.md` (workdir root, scratch).

**Fix 1 — the cursor blink is frame-based in 1.21.11.** 26.1 blinks on
wall time (`TextCursorUtils.isCursorVisible`, `(elapsed / 300) % 2`,
verified from the 26.1 jar); 1.21.11 blinks on a `frame` counter
(`frame / 6 % 2 == 0`, incremented in `tick()`), never reset by typing.
The Rust used the shared `EditBox::cursor_visible()` (time-based 300 ms
like 26.1's form) — but every keystroke resets it via
`insert_text` -> `set_cursor` -> `reset_blink()`. `SignEditor` now
carries `frame: u32`, incremented first in `tick_sign_editor`
(vanilla's order), and the render uses `cursor_shows()`
(`frame / 6 % 2 == 0`). `tick_sign_editor` runs once per client tick
in both the local and remote paths, and `tick_world` runs even when
the world is paused, so the blink runs at 20 Hz while the editor is
open. New test `sign_cursor_blinks_on_the_frame_counter` pins 0–5 on,
6–11 off, 12 on.

**Fix 2 — cursor rendering (bundled version-neutral corrections).**
Both versions draw the cursor identically (verified from the 26.1 jar's
`TextCursorUtils`): an underscore at the end of the line, a 1-wide bar
from `y - 1` to `y + lineHeight` mid-line. The Rust drew a bar from `y`
to `y + lineHeight` in both cases. Now: `"_"` at end-of-line, fill from
`ly - 1.0` with height `line_height + 1.0` mid-line.

Already 1.21.11-shaped, no change: no IME preedit support anywhere in
the editor (26.1-only `preeditOverlay` / `preeditUpdated` /
`textInputManager`); the layout math (title at 40, hanging background
16x16 scaled 4.5 at (0, -13), y offsets 90/125, line heights 10/9, max
line widths 90/60, selection highlight geometry); the applicator path
(`use_sign_applicator` matches `SignBlock.useItemOn` — honeycomb waxes
any sign + event 3003, dyes/ink need text, colour/glow gates, then
click commands, stat, game event, consume); `use_sign_empty_hand`
matches `useWithoutItem`; `TextFieldHelper` starts the cursor at the
end (verified from the 1.21.11 jar); line moves `& 3` with cursor to
end; the width gate `font.width(s) <= maxTextLineWidth` with
revert-on-overflow. The `SignApplicator` signature change (26.1 passes
the held `ItemStack`, 1.21.11 doesn't) is behaviour-free.

Known version-neutral gaps, recorded not fixed: standing-sign editor
text is drawn unscaled where vanilla scales it 0.9765628 (hanging is
1.0 — exact); `SignEditScreen` itself + the standing-sign background
are item 246's scope. Level event 3003 plays `HONEYCOMB_WAX_ON` in
vanilla's `LevelEventHandler` (verified from the 1.21.11 jar) but the
Rust's `block_particle_event` spawns particles only, so a LAN guest
hears no wax sound (the local player hears it via the direct play).
`use_sign_applicator` / `use_sign_empty_hand` don't check
`otherPlayerIsEditingSign`. The Rust triggers `item_used_on_block`
after a sign applicator use; vanilla's `SignBlock.useItemOn` doesn't
— not verified against the jar's trigger callers.

Verification, Nov 20, 2026: `cargo check -p horizon-client --lib
--tests --no-default-features` clean (JOBS=1, 2m26s); `cargo test -p
horizon-client --lib --no-default-features game::sign_edit::` 3 passed
(5m13s, link survived); rustfmt --check clean on sign_edit.rs.
Rank 2 now 240/492 (70 changed, 170 no-change).
work-items.json rank2.closed = 240 on 2026-11-20.

## Item 62 — crates/horizon-client/src/mobs/armadillo.rs (rank 2, CHANGED)

12 classes, 36 methods, 6 named. Javap diff +280/-236 (673 lines),
decompile diff 767 lines, both read in full Nov 20, 2026. Triage note
`rank2-item-62-TRIAGE.md` (workdir root, scratch).

The Java delta is the 26.x Brain-construction refactor only: 26.1 builds
the brain from a static `BRAIN_PROVIDER` + `ArmadilloAi.getActivities()`
(`ActivityData`), 1.21.11 from `ArmadilloAi.brainProvider()` +
`makeBrain(Brain)` (`addActivity` / `addActivityWithConditions`). The
activity contents are identical — same 5 sensors, same 15 memory types,
same core/idle/scared behaviours with the same parameters. The Rust
models the brain as goals + the `State` struct and cites no
`getActivities` / `ActivityData` / `Brain$Packed`, so the refactor
touches no Rust code. Item 38's carry-forward (check ArmadilloAi for
the IS_TEMPTED conflation) is discharged: the armadillo uses the Tempt
goal, no IS_TEMPTED memory is modelled or read here.

**Fix — `sulfur_cube_hot` is not in `#panic_environmental_causes`.**
The 1.21.11 jar's tag has seven entries (cactus, freeze, hot_floor,
in_fire, lava, lightning_bolt, on_fire), verified from the jar and from
the prepared 1.21.10 fixture; there is no `sulfur_cube_hot` damage type
in 1.21.11 (83 damage types, none sulfur) nor in 26.1. But
`armadillo::environmental()` listed it as an eighth member, and three
tests asserted it. Removed the arm (with a comment citing the 1.21.11
tag); the tag test now expects the seven real entries; the panda and
polar-bear `freeze_panic_tests` now use `HotFloor` as the second cause
(renamed `freezing_and_a_hot_floor_are_environmental`). No reachable
behaviour changes: no `sulfur_cube_hot` damage can occur in 1.21.11
vanilla play. The failure was latent — the tag test early-returns
without fixtures, and CI only runs `cargo test -p horizon-protocol`.

Verified against the jars, no change: `isScared` = state != IDLE;
`shouldSwitchToScaredState` = ROLLING and `inStateTicks > 10`;
`canStayRolledUp`; `rollUp`/`rollOut` (stop, reset love, ENTITY_ACTION,
sounds); hurt halving; `actuallyHurt` (danger 80 + roll up on a living
attacker, roll out on environmental damage); the scare sensor — a
`MobSensor(5, isScaredBy, canStayRolledUp, DANGER_DETECTED_RECENTLY, 80)`
built by a `SensorType` lambda (REPORT GAP, in no report item): scans
every 5 ticks, erases the memory when the armadillo can't stay rolled
up, sets it for 80 ticks on a scare, and does NOT erase on a failed
scan; `pickNextScuteDropTime` = `nextInt(6000) + 6000`; the
`ArmadilloBallUp` peek/unroll logic; `mobInteract` (brush, 16
durability, FAIL when scared); `ArmadilloPanic` (roll out on start);
save/load (`state` name + `scute_time`, `orElse(IDLE)`). `undead()` is
exactly the 1.21.11 tag (17: `#skeletons` incl. parched + `#zombies`
incl. camel_husk and zombie_nautilus + wither + phantom — all three
ARE 1.21.11 mobs, verified from the jar's tags).

Known version-neutral gaps, recorded not fixed: the Rust gates the
last-attacker scare check on 100 ticks where vanilla reads
`getLastHurtByMob()` with no time gate; `server_ai_step` adds a
`!liquid` condition to the BallUp start (vanilla checks only
`onGround()`); the sensor's first-scan phase is random in vanilla,
first-tick in the Rust; `ShedScute` plays the sound even if the loot
table yields nothing (unobservable with vanilla data). REFERRALS (same
26.3-era sulfur assumption, other modules): `game/dragon.rs::
is_no_knockback_type` includes `sulfur_cube_hot` and omits freeze,
stalagmite, outside_border, generic_kill, campfire, spear (1.21.11 tag
has 29 entries, verified from the jar); `mobs.rs::Damage::is_fire`
includes `SulfurCubeHot` and omits unattributed_fireball;
`mobs.rs::Damage::no_knockback` includes `SulfurCubeHot` and omits
player_explosion / bad_respawn_point / ender_pearl / dragon_breath
(verify against the enum + the `Named` path before fixing).

Verification, Nov 20, 2026: `cargo check -p horizon-client-lib --lib
--tests --no-default-features` clean (JOBS=1, 40s); `cargo test -p
horizon-client-lib --lib --no-default-features mobs::armadillo::` 10
passed; `... freezing_and_a_hot_floor_are_environmental` 2 passed;
rustfmt --check clean on armadillo.rs, panda.rs, polar_bear.rs.
Rank 2 now 241/492 (71 changed, 170 no-change).
work-items.json rank2.closed = 241 on 2026-11-20.

## Item 63 — crates/horizon-client-render/src/hud.rs (rank 2, NO-CHANGE + citation fix)

13 classes, 45 methods, 5 named. Javap diff +346/-409 (1280 lines),
decompile diff 2164 lines, both read in full Nov 20, 2026. Triage note
`rank2-item-63-TRIAGE.md` (workdir root, scratch).

The Java delta: `TextureUtil.writeAsPNG` gains a per-format texel
decoder in 26.1 (RED8/RED8I luminance, RGBA8 `getInt`, DEPTH32 gray)
where 1.21.11 just `getInt`s — but the only caller in scope,
`TextureAtlas.dumpContents`, dumps RGBA8 atlases, where both versions
behave identically; `Font` / `PlainTextRenderable` / `TextRenderable`
take `Matrix4fc` in 26.1, `Matrix4f` in 1.21.11, with visibility-only
changes otherwise (`drawInBatch`, `drawInBatch8xOutline`, `getGlyph`,
`prepareText`, `markBackground`, the 0.03 front depth all identical);
`SpecialGlyphs` visibility only (MISSING is a 5x8 outline in both);
`SubmitNodeCollection` reshapes its submit API (26.1:
`submitBreakingBlockModel` + `List<BlockStateModelPart>`-based
`submitBlockModel`; 1.21.11: `submitBlock` with special-block-model
rendering + `BlockStateModel` + RGB-float `submitBlockModel` +
`RenderType`-bearing `submitItem`, `CameraRenderState` moved from
`state.level` to `state`); `GpuDevice` / `CommandEncoder` /
`RenderPass` are interfaces in 1.21.11, classes in 26.1
(invokeinterface vs invokevirtual, no behaviour change);
`MapTextureManager.updateTextureIfNeeded` null-checks `getPixels()` in
1.21.11 (26.1 would NPE).

Every Rust mirror in hud.rs is already 1.21.11-shaped: `dump_atlas` /
`dump_contents` match `dumpContents` + `dumpSpriteNames` exactly
(RGBA8-only by construction, so the format switch is unobservable);
`text_styled_world` (0.03 front depth), `text_styled_outline_world`
(eight offsets, POLYGON_OFFSET front), the MISSING-glyph 5x8 outline,
`name_tag` (`markBackground`'s rect: x-1, y-1, advance+1, 10) all match
both versions; the submit-API reshaping has no mirror in hud.rs (only
`submitNameTag` is cited, for text layout); the map null-check has no
mirror (atlas slots, not per-map textures). The only edit: the
`MAP_SLOTS` comment's stale "26.3's `MapTextureManager`" -> "Vanilla's"
(both versions keep a texture per map).

Version-neutral notes: `dump_atlas` writes `\n` where vanilla's
`dumpSpriteNames` uses `%n` (`\r\n` on Windows); `name_tag` computes
its background from the total advance where vanilla tracks min/max
over glyphs (equal for a single line).

Verification, Nov 20, 2026: `cargo check -p horizon-client-render
--lib --tests --no-default-features` clean (JOBS=1, 54s); `cargo test
-p horizon-client-render --lib --no-default-features hud::` 89 passed;
rustfmt --check clean on hud.rs. Rank 2 now 242/492 (71 changed,
171 no-change).
work-items.json rank2.closed = 242 on 2026-11-20.

## Item 64 — crates/horizon-client/src/mobs/undead.rs (rank 2, CHANGED)

7 classes, 15 methods, 8 named. Javap diff +166/-197 (517 lines),
decompile diff 1940 lines (includes Zombie.java, Drowned's
superclass), both read in full Nov 20, 2026, plus direct `javap` on
Zombie / ZombieVillager / EntityType / EntityDimensions /
ProjectileUtil from both jars. Triage note `rank2-item-64-TRIAGE.md`
(workdir root, scratch).

**Fix — baby zombie-family dimensions.** 26.1 gives Zombie, Drowned
and ZombieVillager each their own `BABY_DIMENSIONS =
scalable(0.49, 0.99)` (eyes 0.775 / 0.775 / 0.67) plus a
`getDefaultDimensions` override. 1.21.11 drops all three:
`Zombie.BABY_DIMENSIONS = ZOMBIE.getDimensions().scale(0.5)
.withEyeHeight(0.93)` — the adult 0.6 x 1.95 box (verified from
`EntityType`'s static init) halved to 0.3 x 0.975, eyes 0.93 — and
Drowned / ZombieVillager have no `BABY_DIMENSIONS` and no
`getDefaultDimensions` at all in 1.21.11 (verified by `javap -p`),
so they inherit Zombie's. The species table had the 26.x shape
(`Baby::Abs(0.49, 0.98, 0.775/0.67)`) for all three; now
`Baby::Abs(0.3, 0.975, 0.93)` with Husk-style comments (Husk was
already fixed in item 34). New test
`undead::baby_dims_tests::baby_zombies_are_half_the_adult_box` pins
all four zombie-family entries. REPORT GAP: the report's method
comparison misses this — Zombie's change is `<clinit>`-only, and
Zombie is in item 130 only for finalizeSpawn / handleAttributes /
hurtServer / wantsToPickUp; ZombieVillager is in item 186. Both items
will find the dims already fixed here.

Everything else identical: RemoveBlockGoal.tick (vy 0.3 + 3 egg crumbs
at y+0.7, vy -0.3 on even ticks, progress sound every 6, break after
60 with 20 poofs; only the `ItemParticleOption` ctor and the server
gate differ), `getPosWithBlock` / `isValidTarget`, the turtle-egg
sounds (progress 0.5 / 0.9+mob.random*0.2, break 0.7 /
0.9+level.random*0.2), Drowned finalizeSpawn (3% nautilus shell, 50%
zombie-nautilus jockey — the jockey exists in BOTH versions),
spawn rules, equipment, wantsToSwim, travelInWater, performRangedAttack
(aim lower third, arc dist*0.2, 1.6, 14 - 4*difficulty), getWaterPos,
DrownedMoveControl, SwimUp/TridentAttack/GoToBeach goals,
AbstractSkeleton.reassessWeaponGoal (bow in either hand via
`getWeaponHoldingHand`, hard 20 / normal 40) and performRangedAttack.
The Rust mirrors all of it (verified constant by constant).

Known version-neutral gaps, recorded not fixed: the turtle-egg break
sound's pitch is a flat 1.0 in the Rust (vanilla: 0.9 +
level.random*0.2); the egg-crumb `Particles` event carries no spread
or speed (vanilla: (rand-0.5)*0.08 per axis, 0.15); the baby-dims
change also affects the VEHICLE attachment, which the Rust's `Baby`
model doesn't carry. Zombie's method-level differences belong to item
130; ZombieVillager's behaviour to item 186.

Verification, Nov 20, 2026: `cargo check -p horizon-client-lib --lib
--tests --no-default-features` clean (JOBS=1, 33s); `cargo test -p
horizon-client-lib --lib --no-default-features mobs::undead::` 10
passed (incl. the new baby-dims test); `... baby_` 25 passed;
rustfmt --check clean on species.rs and undead.rs. Rank 2 now
243/492 (72 changed, 171 no-change).
work-items.json rank2.closed = 243 on 2026-11-20.

## Item 65 — crates/horizon-client/src/game/sculk.rs (rank 2, CHANGED)

5 classes, 11 methods, 8 named. Javap diff +213/-211 (601 lines),
decompile diff 1989 lines, both read in full Nov 20, 2026, plus
direct `javap` on LiquidBlock / FlowingFluid / BubbleColumnBlock /
SoulSandBlock / MagmaBlock from the 1.21.11 jar. Triage note
`rank2-item-65-TRIAGE.md` (workdir root, scratch).

**Fix — postProcessGeneration skips LiquidBlocks in 1.21.11.**
Both versions tick the fluid state first; then 26.1 calls
`blockState.tick` on LiquidBlocks (and skips their shape update)
while 1.21.11 skips them entirely. The mirror is
`Game::postprocess_chunks` in `game/postprocess.rs` (not sculk.rs);
it had the 26.1 form. Now liquids get only the fluid tick and the
shape update is gated on `!liquid_block`, and the module doc's
"raises bubble columns" (a 26.1 behaviour) is reworded. Removing
the liquid block tick does NOT lose bubble columns, verified from
the 1.21.11 jar: `LiquidBlock.tick` only schedules a fluid tick;
columns are created by `BubbleColumnBlock.updateColumn`, called by
SoulSandBlock/MagmaBlock, whose `updateShape` schedules a 20-tick
check when water is above — and magma/soul sand are not liquids,
so they DO get the shape update during post-processing. The Rust
already mirrors that path (`shapes.rs` Magma/Liquid arms,
`liquid_block_changed`).

Everything else identical: SpawnUtil.trySpawnMob
(`getRandom()` -> `random` field only; tries, random offsets, world
border, moveToPossibleSpawnPosition, collision check, create,
spawn rules + obstruction, addFreshEntityWithPassengers, ambient
sound, discard), RedStoneOreBlock.spawnParticles (0.5625 offset,
one red dust per non-solid-render face, x/y/z random order),
SculkSensorBlock (water check `is(WATER)` -> `getType() == WATER`,
`Entity.is(WARDEN)` -> `getType() == WARDEN`; shape column(16, 0,
8); resonance tone map [0,0,2,4,6,7,9,10,12,14,15,18,19,21,22,24];
active 30 ticks, cooldown 10, clicking pitch `rand*0.2+0.8`;
getAnalogOutputSignal = ACTIVE ? last frequency : 0; 5 XP),
SculkShriekerBlock (same water-check change only; stepOn player-only
tryShriek; tick SHRIEKING -> false + tryRespond; 5 XP), and the
rest of LevelChunk (visibility/rename only). The sculk.rs mirrors
were verified constant by constant: `sensor_signal`, `activate_sensor`
(resonate 1..=15), `sculk_step_on` (warden excluded, canActivate
gate), `try_shriek` / `try_to_warn` (24-block warden box) / `shriek`
(90 ticks) / `shrieker_respond`, `try_summon_warden` (20 attempts,
`nextInt(11)-5`, y+6, range 6, ON_TOP_OF_COLLIDER, yaw +
follow-range bonus + 5% left-handedness draws, obstruction check),
`spawn_position`, `redstone_ore_particles`, `step_on_redstone_ore`,
`use_redstone_ore`, and the horizon-world constants (SENSOR_ACTIVE
30, CALIBRATED_ACTIVE 10, SENSOR_COOLDOWN 10, SHRIEKING 90).

Version-neutral gaps (recorded, not fixed): `use_redstone_ore`
returns PASS for any held block item, where vanilla also requires
`BlockPlaceContext.canPlace()`; `try_summon_warden` checks
obstruction but not `checkSpawnRules` (below-block `isValidSpawn`);
`redstone_ore_particles` early-returns for a LAN guest.

Verification: `cargo check -p horizon-client --lib --tests
--no-default-features` clean (JOBS=1, 52s); `cargo test -p
horizon-client --lib --no-default-features game::sculk::` 4 passed,
3 ignored (fixtures); rustfmt --check clean on postprocess.rs. No
new test: no harness exercises `postprocess_chunks` (needs a live
Game with lit chunks). Rank 2 now 244/492 (73 changed, 171
no-change).
work-items.json rank2.closed = 244 on 2026-11-20.

## Item 66 — crates/horizon-client/src/game/stats_screen.rs (rank 2, CHANGED)

14 classes, 61 methods, 3 named. Javap diff +734/-807 (2052 lines),
decompile diff +292/-308 (865 lines), both read in full Nov 20,
2026, plus direct `javap` on Stats / WidgetSprites /
StatsScreen$ItemStatisticsList from both jars. Triage note
`rank2-item-66-TRIAGE.md` (workdir root, scratch).

**Fixes (all verified against the 1.21.11 jar).** (1) The CUSTOM table
listed `sleep_in_straw_bed`, which exists in neither jar (the straw
bed is a 26.2/26.3 addition) — the General tab shows every CUSTOM
entry, so the bogus row would render. Removed; new test
`custom_stats_are_the_1_21_11_set` pins the 77-entry set and order.
The other 77 entries were verified one by one against
`Stats.<clinit>` (string ids + formatters): no mismatches. Trap:
comparing Java FIELD names misleads — the field is `CROUCH_TIME`
but the registered id is `sneak_time`; the Rust's `sneak_time` is
correct. (2) The sort-button background is always the header sprite:
`WidgetSprites(HEADER, SLOT)` gives the focused variant the header
too and the button is never inactive; the Rust drew SLOT normally
and HEADER on hover (inverted). (3) The sort arrow sits 18px left
of its button (`getColumnX - 36`); the Rust drew it on the button.
(4) Citations: `ItemRowWidget.extractTooltip` -> `renderTooltip`
(the 1.21.11 name); the test comment's "vanilla 26.3's" ->
"vanilla's".

Java delta: Tooltip loses its 26.1-only `style` + `component`
fields, 3-arg `create` and accessors (no Rust mirror exists —
`ui::tooltip` draws pre-split lines directly); StatsScreen is the
26.x render-state rename throughout (`extractRenderState` ->
`render`, `extractContent` -> `renderContent`, `extractTooltip` ->
`renderTooltip`, `graphics.text` -> `drawString`, etc.) plus
visibility changes and `updateWidth` -> `setWidth` +
`arrangeElements` — no behaviour change. Verified unchanged: the
General tab (sort by translated label, colours, formatters), the
Items tab (columns, header sprites, mined/"-" rules, row shading),
the sort (`sortByColumn` desc -> asc -> off, comparator with -1 for
non-BlockItem on a block column, tie-break by `Item.getId`), the
Mobs tab (texts, argument orders, colours, row 36), the tab-active
rule and the empty-tab tooltip split at 170, the list top (the
ctor's y0=33 is pre-layout; `doLayout` sets it to the 24px header).

Version-neutral gaps (recorded, not fixed): mob rows iterate the
BTreeMap stats values (lexicographic, killed then killed_by) where
vanilla iterates the entity registry — no entity registry order
exists in the Rust (`mob_types()` is a HashMap), so fixing needs
new plumbing; the item slot/tooltip sit 1px off vanilla's anchor
(consistent shift, presumably the renderer's own anchoring); the
item rows' initial order is map order where vanilla uses an
identity hash set (both effectively arbitrary).

Verification: `cargo test -p horizon-client --lib
--no-default-features game::stats_screen::` 6 passed (5 existing +
the new CUSTOM test); rustfmt --check clean on stats_screen.rs.
Rank 2 now 245/492 (74 changed, 171 no-change).
work-items.json rank2.closed = 245 on 2026-11-20.

## Item 67 — crates/horizon-client/src/game/armor_stands.rs (rank 2, CHANGED)

7 classes, 20 methods, 7 named. Javap diff +174/-198 (539 lines),
decompile diff +556/-620 (1658 lines), both read in full Nov 20,
2026, plus direct `javap` on EnchantmentEffectComponents /
EquipmentClientInfo$LayerType / ModelLayers / ArmorStand
(`<clinit>` defaults) and asset listings from both client.jars.
Triage note `rank2-item-67-TRIAGE.md` (workdir root, scratch).

**Fixes (all verified against the 1.21.11 jar).** (1) The stand
texture: 26.1 `textures/entity/armorstand/armorstand.png` vs
1.21.11 `wood.png` (same 809-byte image, renamed; each jar has
only its own file). The Rust's `uv_of("entity/armorstand/armorstand")`
returned None in 1.21.11, so the stand's body never rendered.
Now `entity/armorstand/wood` (body + emissive overlay). (2)
`brokenByAnything` drops ALL equipment in 1.21.11 — the
`PREVENT_EQUIPMENT_DROP` (vanishing) check is 26.1-only (the
component exists in both versions; only the stand's use is new).
Removed the filter from `broken_armor_stand_equipment` (the
player-hit path already dropped everything); test rewritten to
assert vanishing items drop. (3) `HumanoidArmorLayer` has no
`HUMANOID_BABY` layer type in 1.21.11 (absent from the enum and
the assets; 26.1 added it, and skips trims for it). The mirror is
in `mobs/render.rs`: `humanoid_layer_folder` dropped its baby
branch — babies use `humanoid`/`humanoid_leggings` like adults
(only the model is the baby one); test rewritten. The
player/stand armor layers in armor_stands.rs were already
1.21.11-shaped (no baby kind, trims unconditional).

Java delta otherwise: `ArmorStandRenderer.getRenderType` for a
marker is `entityCutoutNoCull` in 1.21.11 vs `entityCutout` in
26.1 (no Rust mirror — `put` has no per-entity cull mode);
`ArmorStand.interact` is `interactAt` in 1.21.11 with a PASS
fallthrough instead of `super.interact` (same outcome);
`Equippable.canBeEquippedBy` takes an `EntityType` in 1.21.11
vs a `Holder` in 26.1 (same result); `CameraRenderState` moved
package. ArmorStandArmorModel, AbstractSkullBlock identical.
Verified unchanged: `interactAt` (name-tag, marker-by-targeting,
disabled/hand-without-arms FAIL, `getClickedSlot` ranges, swap
bits 0/8/16), `hurtServer` (griefing/invulnerability/explosion/
ignite/burn gates, creative one-hit, 5-tick wobble, second-hit
break), default poses (verified from `<clinit>`: head/body 0,
left arm -10/0/-10, right arm -15/0/10, left leg -1/0/-1, right
leg 1/0/1), the wiggle (`sin(wiggle/1.5*PI)*3.0` under 5 ticks),
`swapWithEquipmentSlot` (binding gate, same-item gate, stat,
split arms), skull animation (dragon jaw + piglin ears only),
`ELYTRA_BABY` (exists in both jars).

Version-neutral gaps (recorded, not fixed): the marker
cutout-vs-no-cull distinction has no Rust mirror; mob armor
trims are not rendered at all (`humanoid_armor` has no trim
handling, though vanilla renders them on mobs); the name-tag
branch names the stand inline where vanilla returns PASS.

Verification: `cargo test -p horizon-client --lib
--no-default-features game::armor_stands::` 10 passed, 3 ignored
(fixtures); `cargo test -p horizon-client-lib --lib
--no-default-features mobs::render::tests::a_baby_zombies_armour_uses_the_adult_textures`
passed; rustfmt --check clean on armor_stands.rs (render.rs flags
one pre-existing drift at line 914, left alone). Rank 2 now
246/492 (75 changed, 171 no-change).
work-items.json rank2.closed = 246 on 2026-11-20.

## Item 68 — crates/horizon-client/src/game/perspective.rs (rank 2, CHANGED)

5 classes, 30 methods, 6 named. Javap diff +249/-138 (594 lines),
decompile diff +334/-335 (884 lines), both read in full Nov 20,
2026, plus direct `javap` on BubbleColumnBlock / MagmaBlock /
LiquidBlock / Entity / EntityRenderer / LivingEntityRenderer /
AbstractClientPlayer / InventoryScreen from both jars. Triage
note `rank2-item-68-TRIAGE.md` (workdir root, scratch).

**Fix 1 — SoulSand bubble-column arms.** 1.21.11's SoulSandBlock
(and MagmaBlock) has `tick` (updateColumn above), `updateShape`
(UP becomes water -> schedule self 20 ticks) and `onPlace`
(schedule self 20 ticks); 26.1 has none of the three — the
mechanism moved to LiquidBlock in 26.x (water-side:
`shouldBubbleColumnOccupy` + `tryScheduleBubbleBlockColumn`,
which the Rust mirrors). The Rust had the full 1.21.11-shaped
mechanism for Magma but NO SoulSand arms at all
(`Class::SoulSand` existed and was never matched). Player
placement was covered by the water side, but worldgen /
postprocess soul sand under water never raised a column
(postprocess calls the shape update on non-liquids and skips
the water above — and the item-65 comment claimed the columns
come from "those blocks' own shape update", true for magma,
false for soul sand). Added the three arms in
horizon-world/src/level/{hooks,shapes,tick}.rs, mirroring Magma
exactly; the extra tick is a no-op where the water side already
handled it (`bubble_column_update_for` is idempotent). New test
`soul_sand_under_water_raises_a_bubble_column_from_its_own_shape_update`
pins the discriminating case (raw placement + shape update,
column `drag=false` after 20 ticks; fails without the arms).
**Fix 2 — citations** in perspective.rs:
`extractEntityInInventoryFollowsMouse` ->
`renderEntityInInventoryFollowsMouse` (module doc, fn doc, scale
comment; 1.21.11's InventoryScreen has only that method).

Java delta otherwise: InventoryScreen is the 26.x render rename
throughout, plus one real difference — 1.21.11's doll helper
sets `lightCoords = 15728880` (full bright); the Rust already
passes its packed full-bright `240 | 240 << 8` to every doll
call, so no fix. AvatarRenderer: layers, arm poses, cape/flight
state, scale 0.9375, crouch offset, hand render, rotations,
upside-down all identical; the scoreText difference is
structural — 26.1's `EntityRenderer` sets `scoreText =
entity.belowNameDisplay()` for EVERY entity within 10 blocks
(26.1's `Entity` has `belowNameDisplay`), 1.21.11 only in
`AvatarRenderer` (players only; `Entity.belowNameDisplay`
doesn't exist). The Rust's names.rs is already 1.21.11-shaped
(players only, < 100, lift `9.0*1.15*0.025`). IronBarsBlock:
only the equivalent water-check rename. GuiEntityRenderer:
identical except the CameraRenderState package move.
Verified unchanged: doll math (rect/size/offset/angles),
arm-pose tree + two-handed gate, cape flap/lean/lean2 formulas,
visual shapes (bars empty, soul sand full), `bars_attach_to`
and `is_exception_for_connection`, the bubble-column block's
own 5-tick re-evaluation.

Version-neutral gaps (recorded, not fixed): the avatar crouch
render offset (`scale * 2/16` down, on top of the crouch pose)
has no mirror in `base_transform` — a crouching player renders
0.125 blocks high; `affects_offhand` also suppresses the
off-hand pose for THROW_TRIDENT where vanilla's gate is
`isTwoHanded()` only (nil visible difference).

Verification: `cargo test -p horizon-world --lib
--no-default-features level::tests::soul_sand_under_water_raises_a_bubble_column_from_its_own_shape_update`
passed (early-returns locally — no prepared resources on this
VM; runs for real on CI); `cargo check -p horizon-client --lib
--tests --no-default-features` clean; rustfmt --check clean on
perspective.rs, hooks.rs, shapes.rs, tick.rs (tests.rs flags two
pre-existing drifts, left alone). Rank 2 now 247/492 (76 changed,
171 no-change).
work-items.json rank2.closed = 247 on 2026-11-20.

## Item 69 — crates/horizon-client/src/mobs/body_anim.rs (rank 2, CHANGED)

19 classes, 48 methods, 4 named. Decompile diff +686/-328 (1293
lines) read in full Nov 20, 2026; javap diff +2760/-799 (3889
lines) scanned, key constants confirmed in it (polar bear
`ldc float 0.44444445f`); 26.1's BabyGoatModel,
Adult/BabyFoxModel, Adult/BabyStriderModel, AdultBeeModel and
1.21.11's Rabbit / ModelPart / PartPose / LivingEntityRenderer
decompiled directly. Triage note `rank2-item-69-TRIAGE.md`
(workdir root, scratch).

The 26.x model split is the whole story: 10 of the 19 classes
are 26.1-only Adult/Baby model subclasses that do not exist in
1.21.11. In 1.21.11 each species has ONE concrete model class
whose `setupAnim` serves both ages, and the baby layer is the
adult mesh through a `BABY_TRANSFORMER` (`MeshTransformer.scaling`
0.5 bee/strider, 0.6 armadillo; `BabyModelTransform` fox/goat/
polar bear/rabbit; `LlamaModel::transformToBaby` llama). The
Rust was ported from 26.x, so every baby branch in it mirrors a
26.1 subclass. Seven fixes:

**Fix 1 — polar bear cub head.** 1.21.11 multiplies the head's
stand offsets by `isBaby ? 0.44444445 : 1.0` (4/9 = 1 /
`BABY_HEAD_SCALE` 2.25); 26.1 has no baby factor. The Rust
applied the full offsets to cubs. Body/legs already used
`ageScale`.
**Fix 2 — strider baby branch.** The Rust's baby branch (body
17.25, legs 20.0, `bristle0/1/2` on xRot) is 26.1's
`BabyStriderModel`, decompiled to confirm. Deleted: babies run
the adult animation on the 0.5-scaled baby layer, as in
1.21.11. (Vanilla's absolute body.y = 2.0 / leg.y = 8.0 on the
transformer-shifted baby mesh means the baby strider's feet
don't touch the ground in 1.21.11 either — the port mirrors the
code, floating feet and all.)
**Fix 3 — goat baby tilt.** The PI/8 head tilt is 26.1's
`BabyGoatModel.setupAnim`; removed.
**Fix 4 — bee antennae.** 1.21.11's one `BeeModel` animates the
antennae at every age; the Rust gated them on `!s.baby`
(26.1's base `BeeModel` has no antenna fields — they live on
`AdultBeeModel`).
**Fix 5 — armadillo.** 1.21.11 applies the walk animation
unconditionally (26.1 gates it on `!isHidingInShell`), and one
`ArmadilloAnimation` serves both ages — `BabyArmadilloAnimation`
is 26.1-only (verified absent from the 1.21.11 jar), so the baby
prefix resolved to nothing and baby armadillos never animated
at all.
**Fix 6 — rabbit procedural hop.** 1.21.11 has NO keyframe
animations for rabbits: head always follows the look; haunches
`+= sin(jumpCompletion*PI) * 50°`, front legs `+= sin * −40°`.
Rewrote `rabbit()` to that form, with a new
`rabbit::jump_completion` helper on the client hop counters
(mirroring `Rabbit.getJumpCompletion`). The 26.1-only
`rabbit::base_tick` (`setupAnimationStates` driver) and its
`IDLE_TIMEOUT` clock are deleted with both production callers
(`mobs/ai.rs`, `game/remote.rs`); 1.21.11's `Rabbit` entity has
no `hopAnimationState` / `idleHeadTiltAnimationState` /
`idleTimeout` (javap). Tests rewritten; the idle-timeout test
deleted.
**Fix 7 — fox single form.** Rewrote `fox()` to 1.21.11's
inline `setupAnim`: procedural leg swings at every age (the
`FoxBabyAnimation` walk is gone), crouch/sleep/sit unified
under `ageScale`, kit sleep tail −2.1816616 + `body.z += 2`,
kit sit head −1.75/−0.375, crouch wiggle 0.05 -> **0.01**, and
the pouncing pose deleted (no such pose in 1.21.11 — it was
26.1's `AdultFoxModel.setPouncingPose`). The `pouncing` field
is removed from the `Fox` struct and `fox::pose` (the entity
`POUNCING` flag stays — item 33's pounce pitch uses it).

Harness: `entity_anim_fixture.rs` updated for the new
signatures (rabbit reads `num("jumpCompletion")`); the
entity-model driver's rabbit cases are now `jumpCompletion`
0.5 / 0.25 / 0.0 — the old `hopAnimationState` /
`idleHeadTiltAnimationState` cases would throw in the generic
reflective oracle against 1.21.11's `RabbitRenderState`, which
has no such fields. Fox `isPouncing` cases stay (the field
exists on the render state; both sides now ignore it).

Already 1.21.11-shaped: `llama()`; the strider adult branch
(identical to 1.21.11's inline code and to 26.1's
`AdultStriderModel.customAnimations`); the fox adult
walk/crouch/sleep/sit/faceplant/head-look values; the bee
wing/leg bob and roll; the armadillo head clamps and roll/peek
names. `bat`, `frog`, `parrot`, `snow_golem`, `silverfish`,
`endermite` are not in this item's class list; untouched.

Referrals (base models live in other items): axolotl baby
keyframe branch -> item 779 (`AxolotlModel`); panda baby sit
branch -> item 115 (`PandaModel`); turtle egg-belly baby gate
-> item 152 (`TurtleModel`, + 707 `BabyTurtleModel`). The
`Anim::Rabbit` state slots are now never started for rabbits;
the slots are generic per-mob state and stay.

Verification: `cargo check -p horizon-client --lib --tests
--no-default-features` clean (1m09s); `cargo check
-p horizon-client-lib --example client_probe
--no-default-features` clean (55s); rustfmt --check clean on
body_anim.rs, rabbit.rs, fox.rs, ai.rs, remote.rs,
entity_anim_fixture.rs. New `body_anim::tests`: 6 passed
(baby polar bear head 4/9, baby strider = adult anim, baby fox
sleep/sit at half scale, crouch wiggle 0.01, baby bee antennae,
baby goat no tilt). `mobs::rabbit::`: 10 passed; `mobs::fox::`:
7 passed; `mobs::polar_bear::`: 3 passed. Rank 2 now 248/492
(77 changed, 171 no-change).
work-items.json rank2.closed = 248 on 2026-11-20.

## Item 70 — network (`FriendlyByteBuf` / `Connection` / `HashedStack`) (rank 2, NO-CHANGE)

4 classes, 88 methods, 0 named in Horizon. Javap diff read in
full Nov 20, 2026 (303 lines); decompile diff scanned (its bulk
is Vineflower dropping parameter names on the 1.21.11 side).
Triage note `rank2-item-70-TRIAGE.md` (workdir root, scratch).

Java delta is renames plus two method families the Rust already
has: `Connection` is lambda renumbering + one field visibility
change; `Connection$2` drops a compiler-generated null check;
`FriendlyByteBuf` renames `ChunkPos.unpack`/`pack` to the ctor /
`toLong` (same wire format, one long, x-low/z-high) and ADDS
`readVec3`/`writeVec3` (three doubles) and
`readLpVec3`/`writeLpVec3`, which 26.1 lacks; `HashedStack`
renames `typeHolder()` -> `getItemHolder()`. Rust side verified:
`buf::Vec3d` (three doubles), `buf::ChunkPos` (x-low/z-high),
and `buf::LpVec3` — compared line by line against a fresh
decompile of 1.21.11's `LpVec3` (15-bit components, 2-bit scale
+ continuation VarInt, sanitize, pack/unpack, zero fast-path,
 3.051944088384301E-5 threshold — all match). The pin packet
codecs already use `LpVec3` for AddEntity and SetEntityMotion
velocities. No Rust edits.

Pre-existing failures recorded (not this item's): `cargo test
-p horizon-protocol --lib` is 438 passed / 2 failed, both
network-environment tests untouched since the era-opening
publish — `connection::tests::a_dead_address_does_not_hold_up_the_next`
and `resolver::dns::tests::asks_each_server_in_turn_and_retries_truncated_replies_over_tcp`
— failing in this VM's sandboxed network. `buf::` tests: 9
passed. Rank 2 now 249/492 (77 changed, 172 no-change).
work-items.json rank2.closed = 249 on 2026-11-20.

## Item 71 — crates/horizon-client-render/src/audio.rs (rank 2, CHANGED)

6 classes, 27 methods, 6 named in Horizon. Javap diff read in
full Nov 20, 2026; decompile diff read in full (1394 lines).
Triage note `rank2-item-71-TRIAGE.md` (workdir root, scratch).

Java delta is the 26.1 device-tracking refactor undone:
`DeviceTracker` / `DeviceList` / `SoundBufferLibrary$DebugOutput`
are 26.1-only; `Library` goes back to `init(String, boolean)` with
a stored `defaultDeviceName`, a live `getCurrentDeviceName()`
("Unknown" fallback), `hasDefaultDeviceChanged()`, and
`openDeviceOrFallback(String)` (preferred -> default -> null);
`getChannelDebugString` is `getDebugString` in 1.21.11 (same
"Sounds: %d/%d + %d/%d" format). `SoundEngine.shouldChangeDevice`
is 1.21.11's async 1000 ms CAS state machine
(`DEFAULT_DEVICE_CHECK_INTERVAL_MS`, `devicePoolState`), with NO
"current device has disappeared" branch (26.1 has one).
`SoundBufferLibrary.enumerate` and
`SoundEngine.getSoundCacheDebugStats` are 26.1-only. The preview
switch differs only in the neutral category: 26.1 plays the
CLASSIC cow variant's ambient via `COW_SOUNDS`; 1.21.11 plays
`SoundEvents.COW_AMBIENT` directly.

Rust side was already 1.21.11-shaped in the device logic:
`should_change_device` probes async once per 20 ticks, has no
disappeared branch, and its default/preferred branches match
1.21.11; `open_named` follows the preferred -> default -> null
chain; channel counts are the shared 247 static / 8 streaming;
the preview table already plays `minecraft:entity.cow.ambient`
for neutral. The real change: 1.21.11 has no `DebugEntrySoundCache`
(verified absent from its client.jar), so the F3 sound-cache line
— default-on, backed by `cache_stats()` citing the 26.1-only
`enumerate` — is removed: the `sound_cache` entry from
`debug_entries.rs` (ENTRIES 46 -> 45, test count updated), the
match arm from `game/debug_lines.rs`, `sound_cache_line` from
`debug_overlay.rs` (and its test assertion), and `cache_stats()`
from `audio.rs`. The `census()` control-socket diagnostic stays
(Horizon tooling, not a vanilla mirror). Stale 26.1 citations
fixed: `Library.currentDeviceName` ->
`Library.getCurrentDeviceName`, `lastSeenDevices` ->
`Library.defaultDeviceName` / `hasDefaultDeviceChanged`,
`DeviceTracker.tick` -> `DEFAULT_DEVICE_CHECK_INTERVAL_MS`,
`COW_SOUNDS.get(CLASSIC)` -> `SoundEvents.COW_AMBIENT`,
`getChannelDebugString` -> `getDebugString`. An old
`debug-profile.json` naming `sound_cache` stays inert (unknown
names are kept but never looked up).

Verification: `cargo check -p horizon-client-render --lib
--no-default-features` clean; `cargo check -p horizon-client
--lib --tests --no-default-features` clean; `cargo test -p
horizon-client-render --lib --no-default-features sound_preview::`
2 passed; `cargo test -p horizon-client-lib --lib
--no-default-features debug_overlay::` 8 passed and
`debug_entries::` 5 passed; rustfmt --check clean on the five
edited leaf files. Rank 2 now 250/492 (78 changed, 172 no-change).
work-items.json rank2.closed = 250 on 2026-11-20.

## Item 72 — game/potions.rs (Breeze / potions / XP bottle) (rank 2, NO-CHANGE)

7 classes, 17 methods, 7 named in Horizon. Javap diff read in
full Nov 20, 2026 (310 lines); decompile diff read in full (1120
lines); extra manual decompiles in `scratch-vf/m72/` (Breeze both
versions, ProjectileDeflection 1.21.11, Monster both versions,
BreezeAi both versions). Triage note `rank2-item-72-TRIAGE.md`
(workdir root, scratch).

Java delta is API shape plus one random-stream move, all already
1.21.11-shaped in the Rust. Breeze: the 26.1 brain-provider
refactor undone (1.21.11's `BreezeAi.makeBrain` still sets default
FIGHT with the same four sensors, verified from the jar);
`canAttack` -> `canAttackType` (PLAYER or IRON_GOLEM, super checks
via the pipeline); `deflection` `is()` -> `getType() ==` (wind
charges NONE, else REVERSE); `getSoundSource` HOSTILE is redundant
(Monster returns HOSTILE in both versions, verified by javap);
`JUMP_DUST_PARTICLES_AMOUNT = 20` is declared but never used in
1.21.11 (the tick uses the literal 20 in both versions);
path-malus renames (`DANGER_TRAPDOOR`, `DAMAGE_FIRE`). The XP
bottle draws its 3 + d5 + d5 from the LEVEL random in 1.21.11 (the
entity's own in 26.1) — the Rust already uses `world.random`.
Oozing, AreaEffectCloud, BottleItem, PotionItem.useOn and
ThrowablePotionItem differ only by visibility, generics,
`getRandom()` -> `random`, or the `PROJECTILE_SHOOT_POWER`
final-ness (0.5 in both). Rust verified throughout: breeze
malus/targeting/tick particles/deflection (wind charges exempt,
REVERSE with the 170 + jitter*20 yaw turn, BREEZE_DEFLECT sound),
oozing slime count/spawn, cloud tick (wait, radius, five-tick
doses, 0.5 instant factor), mud conversion, bottle fills. No Rust
edits.

Version-neutral gaps recorded (identical in both versions): a
thrown potion deflected by a breeze does not rotate its yaw (the
random is drawn but discarded; unobservable for potions);
`BottleItem.use`'s `mayInteract` check has no Rust mirror.

Verification: `cargo test -p horizon-client --lib
--no-default-features game::potions::` 11 passed, 7 ignored
(fixtures); `cargo test -p horizon-client-lib --lib
--no-default-features mobs::breeze::` 7 passed. Rank 2 now
251/492 (78 changed, 173 no-change). work-items.json
rank2.closed = 251 on 2026-11-20.

## Item 73 — widely named: core (`net/minecraft/core/*`) (rank 2, NO-CHANGE)

23 classes, 85 methods, 0 named in Horizon. Javap diff read in
full Nov 20, 2026 (431 lines); decompile diff read via a
normalizing comparison (its bulk is Vineflower dropping parameter
names and re-flowing lines on the 1.21.11 side); BlockPos$1/$2/$6
checked by javap in `scratch-vf/m73/`. Triage note
`rank2-item-73-TRIAGE.md` (workdir root, scratch).

The real Java delta is 26.1-only data components on holders:
`Holder.Direct`'s components field, `Holder.components()` /
`areComponentsBound()`, `Holder.direct(value, components)`,
`Holder.Reference.bindComponents`, `Registry.componentLookup()`,
plus `WritableRegistry.bindTags(Map)` (1.21.11 has only the
single-tag `bindTag`) and `HolderGetter.Provider.get/getOrThrow(TagKey)`
defaults. NONE of it has a Rust mirror — the Rust has no general
`Holder<T>` (only `trim.rs`'s components-free `Named | Direct`),
no `component_lookup` / `bind_components` anywhere, and the only
`bind_tags` is worldgen's own feature-block binding. The rest is
compiler/visibility noise (Direction lambda order, Axis ctor
visibility, HolderSet$Direct visibility, Registry$1 requireNonNull).
BlockPos, MutableBlockPos, Direction and UUIDUtil are
bytecode-identical between versions (no javap hunks) — the
report's "changed" counts there come from the decompile
comparison. Rust mirrors verified matching: BlockPos pack/unpack,
`between_closed` order (X, Y, Z), `between_corners_in_direction`
(axisStepOrder YZX/YXZ, start-corner sign, NaN negative),
`spiralAround`, Direction 2D/3D data values and
`orderedByNearest`, UUID text/int-array/wire forms. No Rust edits.

Version-neutral gaps recorded (identical in both versions):
`from_3d_data_value` uses `(data % 6).unsigned_abs()`, wrong for
negative data (vanilla floorMods; `from_2d` already uses
`rem_euclid`); `ordered_by_nearest` uses fdlibm trig where vanilla
uses the Mth sine table (cf. item 58); `spiral_around` skips
vanilla's axis validation (unobservable).

Verification: `cargo test -p horizon-core --lib position::` 4
passed; `cargo test -p horizon-core --lib geometry::` 29 passed;
`cargo test -p horizon-client-core --lib uuid::` 2 passed. Rank 2
now 252/492 (78 changed, 174 no-change). work-items.json
rank2.closed = 252 on 2026-11-20.

## Item 74 — environment_timeline.rs (rank 2, CHANGED)

5 classes, 24 methods, 6 named. Javap diff (550 lines) and decompile
diff (573 lines) read in full Nov 20, 2026. Triage note
`rank2-item-74-TRIAGE.md` (workdir root, scratch); oracle probe in
`scratch-vf/m74/`.

The 26.x WorldClock system is 26.1-only: 1.21.11's `Timeline` has no
`clock` and no `time_markers` (codec: `period_ticks` + `tracks` only),
samples at `Level.getDayTime()`, and has no `getPeriodCount`. Fixes in
`crates/horizon-world/src/environment_timeline.rs`:
`Timeline::parse` no longer requires `"clock"` (it failed on 3 of the
4 shipped 1.21.11 timelines — moon, early_game, villager_schedule have
no clock; only day.json carries the vestigial one, which the 1.21.11
codec ignores) and no longer validates `time_markers`; the `clock`
field and the 26.1-only `period_count` are gone. The environment
system (`environment_system.rs`, `environment.rs`,
`environment_system_fixture.rs`) now samples tracks at
`Conditions.day_time: i64` instead of through a clock-name lookup —
both call sites already ignored the name and returned the day time,
so live sampling is unchanged, only the dead 26.x plumbing is gone.
`solve_t` rewritten to vanilla's `CubicBezier.apply`: a plain 4-step
Newton loop (gradient checked first, no step clamp, no bisection) —
the Rust had the 26.3 clamp+bisect shape (`solveT`/`solveTBisect`,
deleted); fixed here because the item's harness compares bit-exact
easing output against the 1.21.11 oracle and `EasingType$CubicBezier`
is in no other item. Verified already correct: the 32-entry EASINGS
table, Curve sample/gradient, cubic_bezier validation, the
validateKeyframes mirror (including the last-ticks-seeded repeat
count), validatePeriod bounds, validateInternal's no-period skip,
`getCurrentTicks` remainder, and the sampler's floor-modulo loop.

Verification: a Java probe ran the real 1.21.11 `CubicBezier.apply`
over the driver's five BEZIERS x 13 xs (65 ground-truth bits,
`scratch-vf/m74/oracle-bits.txt`), pinned in
`cubic_bezier_matches_the_1_21_11_oracle`; a new
`timeline_parse_takes_the_1_21_11_codec_shape` test pins the codec
shape. `cargo test -p horizon-world --lib --no-default-features
environment_timeline::` 3 passed; `cargo check` clean for
horizon-client-render and the client_probe example; rustfmt clean on
all four edited files. Note: the environment_system/timeline harness
drivers still read `.local/horizon-client/26.3/data/minecraft`
(T4/harness-repointing scope). `WorldClock`/`ClockManager` mirrors
remain elsewhere (chat.rs, commands.rs, remote.rs, hub.rs, play.rs,
datafix/schema.rs, world.rs) — their own items' scope. Rank 2 now
253/492 (79 changed, 174 no-change). work-items.json rank2.closed =
253 on 2026-11-20.

## Item 75 — font.rs (rank 2, NO-CHANGE)

37 classes, 53 methods, 3 named. Javap diff (1032 lines) read in
full Nov 20, 2026; decompile diff (3645 lines) scanned via a literal
comparison — every string and numeric literal in all 37 classes is
identical between versions. Triage note `rank2-item-75-TRIAGE.md`
(workdir root, scratch). The Java delta is zero: visibility
(private -> package-private), dropped `Objects.requireNonNull` on
inner-class outer refs, `GpuDevice`/`CommandEncoder` class (26.1) vs
interface (1.21.11), `memFree` overload resolution, `Matrix4fc` vs
`Matrix4f`, the `AtlasManager` package move
(`model/sprite` -> `model`), lambda naming, and a typeSwitch
descriptor — no method body, constant or codec field differs. The
three named methods (`getFontSetRaw`, `getGlyph`,
`TrueTypeGlyphProviderDefinition.load`) are unchanged. Rust mirrors
spot-checked against the 1.21.11 decompile and matching:
unihex parse/`calculateWidth`/advance (left-aligned rows at all
widths, empty-glyph (0, bitWidth), integer-division advance),
`getActualGlyphWidth` (rightmost non-transparent column),
`load_bitmap` (cell math, scale, advance formula, `'\0'` skip,
height 8 / ascent 7 defaults), size_overrides' string-only
`CODEPOINT` form (verified by javap on ExtraCodecs), reference
expansion with filter merging. Data note: `include/unifont.json`
has an EMPTY providers list in both versions and no `unifont.hex`
ships in either jar — unihex only matters for packs.

Version-neutral gaps recorded (identical in both versions):
`fishy_advance` checks one advance where vanilla checks unbolded
AND bolded; duplicate codepoints in one bitmap provider are
last-wins in vanilla, first-wins in the Rust; vanilla rejects
unequal bitmap row lengths and ascent > height, the Rust loads
them. No Rust edits.

Verification: `cargo test -p horizon-client-render --lib
--no-default-features font::` 14 passed. Rank 2 now 254/492
(79 changed, 175 no-change). work-items.json rank2.closed = 254
on 2026-11-20.

## Item 76 — mobs/spawn_rules.rs (rank 2, NO-CHANGE + citation fixes)

5 classes, 13 methods, 7 named. Javap diff (247 lines) read in
full Nov 20, 2026; the substantive methods read from the 1.21.11
decompile. Triage note `rank2-item-76-TRIAGE.md` (workdir root,
scratch). The Java delta is renames plus one real branch change,
all already 1.21.11-shaped in the Rust: `BlockAndLightGetter` is
26.1's name for 1.21.11's `BlockAndTintGetter`;
`getLightDampening` is 26.1's name for `getLightBlock`;
`FIRE_IN_NEIGHBOR`/`FIRE` are 26.1's names for
`DANGER_FIRE`/`DAMAGE_FIRE`; `Animal.mobInteract` checks `isBaby()`
in 1.21.11 where 26.1 checks `canAgeUp()` (which also refuses an
age-locked baby — age locking is 26.x-only). Verified matching:
`constructor_maluses` sets DangerFire 16.0 / DamageFire -1.0 for
animals; `feed_animal`/`takes_food` gate on `(age == 0 &&
breedable) || age < 0` (the age>0 case is refused, so the age-up
arm only runs for babies); `animal_rules`, `surface_water`,
`can_see_sky_from_below_water`, `is_dark_enough`, `bright`,
`slime_rules` all match the 1.21.11 methods exactly, including the
slime surface branch (`getMaxLocalRawBrightness` =
`raw_brightness(sky_darken)`) and the double nextInt(10) slime
chunk. No Rust mirror of the 26.1-only
`getEffectiveSkyBrightness` to remove; `getBlockTint` (1.21.11-only
on LevelReader) has no Rust mirror either. Citation fixes
(comment-only): `BlockAndLightGetter.canSeeSky` ->
`BlockAndTintGetter.canSeeSky` (trident.rs), `getLightDampening` ->
`getLightBlock` (light.rs, mod_content.rs). Not touched:
`LightEngine.getLightDampeningInto` (tick.rs — a LightEngine
method, its 1.21.11 name unverified) and the `dampening` field
name in BlockFacts (a prepared-data key).

Version-neutral gap recorded: `slime_rules` has no `instanceof
WorldGenLevel` guard before the slime-chunk branch (unobservable —
`SpawnLevel` always carries a world seed). No behaviour edits.

Verification: `cargo test -p horizon-client-lib --lib
--no-default-features mobs::spawn_rules::` 2 passed (the
horizon-client binary runs 0 — the tests live in the
horizon-client-lib binary). Rank 2 now 255/492 (79 changed,
176 no-change). work-items.json rank2.closed = 255 on 2026-11-20.

## Item 77 — gfx/core_sky.rs (rank 2, NO-CHANGE + citation fixes)

4 classes, 20 methods, 6 named (`buildEndSky`, `buildStars`,
`renderEndFlash`, `renderSkyDisc`, `renderStars`,
`renderSunriseAndSunset`). Javap diff (1097 lines) read in full
Nov 20, 2026; the substantive methods read from the 1.21.11
decompile; a literal comparison over the decompile confirms the
only literal difference in DefaultVertexFormat is the string
`"Normal"` x1. Triage note `rank2-item-77-TRIAGE.md` (workdir
root, scratch). The Java delta is three real changes plus API
shape, all already 1.21.11-shaped in the Rust:
`DefaultVertexFormat.BLOCK` gains Normal + padding(1) in 1.21.11
(a `<clinit>`-only change the report's method comparison misses,
second case after item 64's Zombie dims; `ENTITY` -> `NEW_ENTITY`
is a same-layout rename) — but 1.21.11's `terrain.vsh` declares
`in vec3 Normal` and never uses it (vertex colour is Colour x
lightmap), so the BLOCK normal is write-only data for terrain and
the Rust's normal-less `TerrainVertex` is observably equivalent
(the entity `Vertex` has a normal); the mesher follow-up from the
brief is discharged, no mesher change. `ColorTargetState` +
`WriteMask` are 26.1-only classes, absent in 1.21.11; no Rust
mirror of Mojang's classes exists (only wgpu's own
`ColorTargetState`). `SkyRenderer.buildStars` uses
`RandomSource.create(10842L)` -> `LegacyRandomSource` in 1.21.11
vs `createThreadLocalInstance(10842L)` ->
`SingleThreadedRandomSource` in 26.1 — different RNGs, different
star fields; the Rust already uses `LegacyRandom::new(10842)`.
The rest is package moves (AtlasManager, SkyRenderState) and
GpuDevice/CommandEncoder/RenderPass interface-vs-class.
`RenderPipelines.SKY` in 1.21.11 (verified from both jars'
`<clinit>` bytecode) has no blend call (no blending is the
builder default) and `withDepthWrite(false)` — 26.1's chain
lacks the depth-write call; the Rust's `Part::new` already sets
depth write off and no blend. `RenderPipelines.STARS` uses
`BlendFunction.OVERLAY` = (SRC_ALPHA, ONE, ONE, ZERO), exactly
the Rust's wgpu overlay. Verified matching: `disc_triangles`
(centre (0,16,0), rim 512, -180..=180 step 45, top disc only, as
`renderSkyDisc` draws it); `star_triangles` (1500 draws,
rejection bounds, normalize x 100, the rotateTowards basis
checked against JOML 1.10.8's bytecode: s = up x f, u = f x s,
and the rotateZ post-multiply); the dark disc
(`frame.dark_disc`, `fs_dark_disc`, `dark_disc()` — flat ->
minY, else 63, Overworld-only, not underwater;
`getHorizonHeight` verified from the jar: `isFlat ? minY :
63.0`); the sunrise fan (apex (0,100,0), rim (sin*120, cos*120,
-cos*40), 16 triangles, Rx(90) then Rz(flip+90), scale(1,1,alpha)
folded into the rim's z in local space, alpha gate > 0.001);
the celestial bodies (sun 30, moon 20, quad at y=100, frame
RotY(-90)*RotX(angle), moon phase order and dayTime/24000%8).
Citation fixes (comment-only): "26.3's ProjMat already has
wgpu's reversed 0 to 1 depth" -> the ProjMat uploaded here (the
production caller passes `world_projection`, glam
`perspective_rh` with near/far swapped) — vanilla 1.21.11's own
ProjMat is JOML's 4-arg `perspective` (-1..1), verified from
`GameRenderer.getProjectionMatrix`, so the comment names the
uploaded uniform, not vanilla's; "RenderPipelines.SKY:
ColorTargetState.DEFAULT, no blending." -> "no blend call (the
builder default), no depth write."

Version-neutral gap recorded: `stars.rs` computes `z_rot` with
f64 `TAU` where vanilla uses `(double)(float)Math.PI * 2.0`
(~1.75e-7 relative on a star's spin angle — sub-pixel,
unobservable in both versions). Referrals: the same stale
"26.3's ProjMat" comment in `gfx/core_lines.rs:25`, and
"ColorTargetState.DEFAULT" citations in `post_chain.rs:426,
808` (their files' items).

Verification: `cargo test -p horizon-client-render --lib
--no-default-features gfx::core_sky::` 2 passed;
`gfx::sunrise_fan::` 4 passed; rustfmt --check clean on
core_sky.rs. Rank 2 now 256/492 (79 changed, 177 no-change).
work-items.json rank2.closed = 256 on 2026-11-20.

## Item 78 — game/advancements.rs (rank 2, NO-CHANGE + citation fixes)

25 classes, 59 methods, 2 named
(`AdvancementToast.extractRenderState`, `ItemPredicate.test`).
Javap diff (1097 lines) read in full Nov 20, 2026; a literal
comparison over the decompile finds ZERO string or numeric
differences in all 25 classes. Triage note
`rank2-item-78-TRIAGE.md` (workdir root, scratch). The Java delta
is API shape only, no behaviour: `validate(ValidationContextSource)`
-> `validate(CriterionValidator)` in all 12 TriggerInstance
classes plus `Advancement.lambda$validate` (same fields, same
param sets, same names validated — 26.1 calls
`Validatable.validate`, 1.21.11 calls
`CriterionValidator.validate/validateEntity/validateEntities`);
`ItemStackTemplate` -> `ItemStack` for `DisplayInfo.icon` and
`Advancement$Builder.display` (icon codec
`ItemStackTemplate.CODEC` -> `ItemStack.STRICT_CODEC`);
`ItemInstance` -> `ItemStack` for `ItemPredicate.test` and the
trigger methods (`count()` -> `getCount()`);
`extractRenderState(GuiGraphicsExtractor)` -> `render(GuiGraphics)`
on Toast/AdvancementToast/RecipeToast (`text` -> `drawString`,
`fakeItem` -> `renderFakeItem`); 26.1's AdvancementToast caches
`iconItem` in the ctor, 1.21.11 reads `display.getIcon()` live at
render. No Rust mirror of ItemStackTemplate, ItemInstance,
ValidationContextSource or CriterionValidator exists in this
file; every game trigger function already takes `&ItemStack`,
and the lib's `Display.icon` is a plain item-id string parsed
from the 1.21.11 JSON form. Verified matching:
`AdvancementToast.render` (DISPLAY_TIME 5000, title split at 125,
challenge -30465 else -256, one-line label (30,7) + title (30,18)
white, multi-line label fade to 1500 ms then lines fading in at
*252 centred on height/2 - n*9/2, icon (8,8));
`RecipeToast.update`/`render` (changed/lastChanged logic, index
`time / max(1.0, 5000*multiplier/n) % n`, title (30,7) -11534256,
description (30,18) -16777216, station at 0.6 scale (3,3),
result (8,8)); TOAST_SLOTS 5, 32 px slots, 160 px width;
`award()` toast gate; `recipe_toast` follows
`RecipeToast.addOrUpdate`. Citation fixes (comment-only):
`extractRenderState` -> `render` in the `draw_toasts` doc and the
`draw_toast` comment. Version-neutral notes: the Rust's `Toast`
snapshots the icon id at award where 1.21.11 reads the display's
icon live (unobservable — displays immutable); vanilla draws the
fading label/lines even at alpha 0, the Rust skips alpha-0 draws
(unobservable).

Verification: `cargo test -p horizon-client --lib
--no-default-features game::advancements::` 13 passed, 8 ignored
(fixtures). rustfmt not run: the file declares `#[path]` test
submodules, and the edits are comment-only. Rank 2 now 257/492
(79 changed, 178 no-change). work-items.json rank2.closed = 257
on 2026-11-20.

## Item 79 — mobs/equine.rs (rank 2, CHANGED)

5 classes, 29 methods, 5 named (`AbstractEquineModel.setupAnim`;
`AbstractHorse.aiStep`, `causeFallDamage`, `handleEating`,
`playStepSound`). Javap diff (1262 lines) read in full Nov 20,
2026; a literal comparison over the decompile. Triage note
`rank2-item-79-TRIAGE.md` (workdir root, scratch). The Java delta
is the 26.x Adult/Baby model split, equine edition (same pattern
as item 69): 26.1 has `BabyHorseModel` + `BabyDonkeyModel` (both
absent in 1.21.11 — verified by jar listing) and a 7-arg
`AbstractEquineModel` ctor; 1.21.11 has one `AbstractEquineModel`
with `BABY_TRANSFORMER` + `createBabyMesh`/`createFullScaleBabyMesh`
and `setupAnim` scaling its offsets by `EquineRenderState.ageScale`
= `LivingEntity.getAgeScale()` = `isBaby() ? 0.5 : 1.0` (verified
from the 1.21.11 jar; no override in AbstractHorse). Scaled: head
y (-8, 7), head z (-4), leg y (12.0), leg z (4.0), tail y/z
(speed, speed*2). NOT scaled: stand angle (PI/12), leg xRot
offset (-PI/3), tail xRot (PI/6 + speed*0.75). 26.1's setupAnim
inlines the same constants via virtual getters with NO ageScale;
the baby classes overrode them: BabyHorseModel (4.0, 0.0, -PI/2),
BabyDonkeyModel (PI/3, 1.0, 0.5, 0.0, -PI/4) — verified from the
26.1 jar. The Rust's (baby, donkey) parameter table was verbatim
those 26.1 overrides. 1.21.11's `DonkeyModel.setupAnim` only calls
super + toggles chest visibility (`hasChest`); `HorseModel` is
just a ctor. Goal registration: 1.21.11's `registerGoals` adds
MountPanicGoal(1) FIRST, then RunAroundLikeCrazyGoal(1), Breed(2),
FollowParent(4), stroll(6), look(7), random look(8),
RandomStand(9); `addBehaviourGoals` adds FloatGoal(0) +
TemptGoal(3, 1.25, HORSE_TEMPT_ITEMS). 26.1 keeps MountPanicGoal in
`addBehaviourGoals` (after RunAroundLikeCrazy).
SkeletonHorse's `addBehaviourGoals` override is EMPTY in both
versions; ZombieHorse's adds Float + Tempt in both. Because
MountPanicGoal moved to `registerGoals` — which neither subclass
overrides — the skeleton AND zombie horse GET MountPanicGoal in
1.21.11 (in 26.1 neither had it). `MountPanicGoal.shouldPanic` =
`!isMobControlled() && super.shouldPanic()` (identical both
versions); `Mob.isMobControlled()` returns false and only
`ZombieHorse` overrides it (`getFirstPassenger() instanceof Mob`).
Sounds: 26.1 plays `HORSE_LAND_BABY` / `HORSE_STEP_BABY` for
babies; 1.21.11 always plays `HORSE_LAND` / `HORSE_STEP`. The
Rust already plays the adult sounds unconditionally. `handleEating`:
food table identical (wheat 2.0/20/3, sugar 1.0/30/3, hay
20.0/180/0, apple 3.0/60/3, red mushroom 3.0/0/3, carrot 3.0/60/3,
golden carrot 4.0/60/5+love, golden apple 10.0/240/10+love);
26.1 checks `Items.HAY_BLOCK`, 1.21.11 `Blocks.HAY_BLOCK.asItem()`
(same item); 26.1 checks `isAgeLocked()` before the baby age-up
arm, 1.21.11 has NO age-lock check (age locking is 26.x-only, cf.
item 76). The Rust's table + `feed_mount` already match 1.21.11.
`CamelHuskRenderer`: 26.1 extends MobRenderer with AdultCamelModel
+ a static saddle layer; 1.21.11 extends CamelRenderer and
overrides `createCamelSaddleLayer` (CAMEL_HUSK_SADDLE +
CAMEL_HUSK_BABY_SADDLE). Same texture; shadow 0.7 in both. The
husk baby saddle layer is built from the SAME mesh + the SAME
`CamelModel.BABY_TRANSFORMER` as the camel baby saddle (verified
in LayerDefinitions) — geometrically identical, so the Rust's
`camel_baby#saddle` geometry + husk texture is correct. `aiStep`:
NO hunks — identical. `BABY_SCALE = 0.7F` is a 26.1-only field;
no Rust mirror.

Fixes (equine.rs): (1) `anim()` to the 1.21.11 ageScale form —
dropped the (baby, donkey) parameter table and the baby-donkey
specials (hind-leg lerp, head-pitch -30 degree override); one set
of constants with `age = if baby { 0.5 } else { 1.0 }`
(`LivingEntity.getAgeScale`): stand angle PI/12, stand_y 12.0*age,
stand_z 4.0*age, stand_x_off -PI/3, tail xRot PI/6 + speed*0.75;
head y `lerp(eating, lerp(standing, 0, -8*age), 7*age)`, head z
`lerp(standing, z, -4*age)`. Donkeys/mules now animate exactly
like horses (only the chest toggle differs, already handled in
anim.rs). (2) `register()` to the 1.21.11 order and gates:
MountPanic(1) FIRST, before RunAroundLikeCrazy(1), for EVERY horse
species; the skeleton keeps no Float/Tempt (empty override), the
zombie keeps Float + Tempt. Observable: a riderless zombie/skeleton
horse on fire now panics, as in 1.21.11. (3) `MountPanic.can_use` /
`can_continue` to `!isMobControlled()` (version-neutral fix,
bundled with the registration change — the zombie horse newly gets
the goal, and its gate was wrong for exactly that species): the
Rust required NO rider at all, so a player-ridden horse never
panicked (vanilla: it does) and a zombie horse with a mob rider did
panic (vanilla: it doesn't). Now reuses the existing
`mob_controlled_rider` helper. Verified matching, no edits: step
sounds (`entity.horse.step` unconditional, gallop/step_wood logic);
land sound (`living.rs`: `entity.horse.land` 0.4/1.0, no baby
branch); food table + `feed_mount` (exact, including the age-up
`* 20` and no age-lock check); donkey chest toggle (anim.rs);
camel husk texture/model/saddle. Version-neutral note: the baby
horse MODEL GEOMETRY (`horse_baby#main` etc.) is baked by the model
pipeline from the layer definitions — the 1.21.11
`createBabyMesh`/`BABY_TRANSFORMER` application is the bake
pipeline's scope; the keys already exist (species.rs).

Verification: `cargo test -p horizon-client-lib --lib
--no-default-features mobs::equine::` 10 passed, 1 ignored
(fixture). rustfmt --check on equine.rs: only pre-existing drift
at line 630 (gallop breathe push, untouched by this item). Rank 2
now 258/492 (80 changed, 178 no-change). work-items.json
rank2.closed = 258 on 2026-11-20.

## Item 80 — border_render.rs (rank 2, CHANGED)

4 classes, 27 methods, 5 named (`WorldBorderRenderer.extract`,
`render`; `Mth.ceil`, `createInsecureUUID`, `floor`). Javap diff
(945 lines) read in full Nov 20, 2026; a literal comparison over
the decompile. Triage note `rank2-item-80-TRIAGE.md` (workdir root,
scratch). The Java delta: `WorldBorderRenderer` is a package move
only — `state/level/WorldBorderRenderState` (26.1) vs
`state/WorldBorderRenderState` (1.21.11); `GpuDevice` /
`CommandEncoder` / `RenderPass` are interfaces in 1.21.11 vs
classes in 26.1; `RenderPass$Draw` takes 6 args in 1.21.11, 7 in
26.1 (an extra `iconst_0` — the literal comparison's only
WorldBorderRenderer delta). The extract/render/rebuild/
shouldRebuild logic is byte-identical modulo the package path.
`WorldBorderRenderState` + `DistancePerDirection`: NO javap hunks,
ZERO literal differences — identical (the report's method counts
are decompile noise, cf. item 73). `Mth`: `floor`/`lfloor`/`ceil`/
`ceilLong` use trunc + adjust in 1.21.11 vs `Math.floor/ceil` +
cast in 26.1 (identical for all finite in-i32-range values; they
differ only out of range, where the 1.21.11 form wraps MIN - 1 to
MAX); `wobble` uses `RandomSource.create` (LegacyRandomSource) in
1.21.11 vs `createThreadLocalInstance` (SingleThreadedRandomSource)
in 26.1 — different deterministic jitter (same pattern as item
77's buildStars), no Rust mirror anywhere;
`createInsecureUUID()` (no-arg) + the static `RANDOM` field are
1.21.11-ONLY (26.1 removed them), no Rust mirror anywhere (the
values are random by design). **RENDERING_OFFSET does not exist in
either version**: no such field in 26.1's or 1.21.11's
WorldBorderRenderer (verified by javap on both jars), no 0.01
literal in either decompile; both versions' rebuild uses the raw
state bounds. The Rust's `OFFSET = 0.01` cited a vanilla constant
that exists in neither version in this diff.

Fix (border_render.rs): removed the RENDERING_OFFSET inset from
`rebuild` — the min/max clamps and the south/east wall positions
now use the raw state bounds, as both versions'
`rebuildWorldBorderBuffer` does; the `OFFSET` const is deleted.
The walls sit exactly on the border lines in 1.21.11 (the Rust had
them inset by 1 cm). Verified matching, no edits: `extract`
(gates, `alpha = 1 - distance/rd`, `pow(alpha, 4)`, clamp, tint);
`closestBorder` (north/south/west/east order, stable sort by
`Double.compare` — the Rust's `java_double_compare` handles NaN
and -0.0/+0.0 exactly); all 16 rebuild vertices (quads, u/v,
vertex order incl. the north quad's reversed u, `(floor(min) & 1)
* 0.5` for u0, `(max - min) / 2` for u1); `prepare` (alpha gate,
`v0 = -frac(camY * 0.5)`, `v1 = v0 + half_height`, rebuild arg
order); `transforms` (colour /255 + alpha, model offset, texture
offset `(millis % 3000) / 3000` on BOTH u and v); `sides` (filter
`distance < renderDistance`, `get2DDataValue` indices);
`shouldRebuild`; `depth_far` — vanilla's call site
(`LevelRenderer.lambda$addWeatherPass$4`) passes
`GameRenderer.getDepthFar()` = `max(renderDistance * 4, cloudRange
* 16)` with renderDistance in blocks, exactly the Rust's
`camera::depth_far`; BorderStatus colours verified from the
1.21.11 jar (GROWING 0x40FF80, SHRINKING 0xFF3030, STATIONARY
0x20A0FF); the Rust's local `floor`/`ceil` already use the 1.21.11
trunc+adjust form (there is no central `jmath::floor`/`ceil`
mirror). Version-neutral notes: the Rust's `floor`/`ceil` use
`v as i32` (Rust saturates) where Java's cast wraps — unobservable
at border coordinates (bounded to ±3e7); the
`world_border_render` parity harness + oracle are 26.3-shaped (the
oracle drives the real 26.3 renderer via reflection, calling
`prepareDynamicTransforms`/`draw` as separate methods; 1.21.11
inlines them in `render`) — removing the offset diverges the Rust
from the 26.3 oracle, expected for the 1.21.11 port, oracle rewrite
is T4 scope.

Verification: `cargo test -p horizon-client-render --lib
--no-default-features border_render::` 1 passed. rustfmt --check
clean on border_render.rs. Rank 2 now 259/492 (81 changed, 178
no-change). work-items.json rank2.closed = 259 on 2026-11-20.

## Item 81 — entities.rs (rank 2, CHANGED)

6 classes, 17 methods, 6 named: ExperienceOrbRenderer,
ItemEntity, PowderSnowBlock, ItemUtils, ServerEntity,
LavaCauldronBlock. The literal comparison over the decompile
finds one string-level delta in all six classes — the orb
texture path — and no real numeric delta (small-int noise only).

Java delta, jar-verified: (1) ItemEntity's 5-arg ctor draws its
spawn motion from `level.random` in 1.21.11 (2 nextDouble, y =
0.2, before the Entity ctor runs); 26.1 draws it from the
item's own random after setItem. The 7-arg ctor order swaps
too (setDeltaMovement before setItem in 1.21.11) — unobservable
in the Rust. UUID source is the entity random in BOTH versions
(`Mth.createInsecureUUID(this.random)`, verified from both jars'
Entity ctor bytecode); bobOffs and yaw are identical. (2)
ItemEntity.tick: 26.1 calls `applyEffectsFromBlocksForLastMovements()`
on idle (resting) ticks; 1.21.11 has NO else branch — nothing
happens on idle ticks. The method does not exist in 1.21.11's
Entity (javap-verified); its 26.1 body is
`applyEffectsFromBlocks(finalMovementsThisTick)`. The move gate
is the exact De Morgan negation in both versions and the move
arm is identical. (3) ServerEntity.sendChanges: the only real
difference is `tickCarriedBy(player, stack, frame)` (26.1) vs
`tickCarriedBy(player, stack)` (1.21.11); 1.21.11's form reads
the frame state from the stack via `ItemStack.isFramed()`
(twice — holder retention and the dimension check). (4)
ExperienceOrbRenderer: texture path 26.1
`textures/entity/experience/experience_orb.png` vs 1.21.11
`textures/entity/experience_orb.png`; RenderType rename
(entityTranslucentCullItemTarget -> itemEntityTranslucentCull).
(5) `onSyncedDataUpdated(EntityDataAccessor)` is 1.21.11-ONLY
(DATA_ITEM change -> `getItem().setEntityRepresentation(this)`).
(6) PowderSnowBlock: `entity.is(tag)` -> `entity.getType().is(tag)`
(same result), `BlockState.is(Object)` -> `is(Block)` (API
shape). (7) LavaCauldronBlock: `CauldronInteractions.LAVA`
(Dispatcher) -> `CauldronInteraction.LAVA` (InteractionMap),
same interactions. (8) ItemUtils.onContainerDestroyed:
Stream -> Iterable, same body (5-arg ctor at the exact
position). (9) Fluid rename `updateFluidInteraction` ->
`updateInWaterStateAndDoFluidPushing` — the Rust already cites
the 1.21.11 name.

Fixes: (a) `add_item_entity` (game/item_entities.rs) now draws
the spawn motion from `self.world.random` (2 nextDouble, y =
0.2) instead of the item's own random — each spawn advances the
shared deterministic level-random stream by 2 draws, and the
item's own random is no longer advanced by the motion draws
(later item-random draws, e.g. the escape speed, would
otherwise shift). All `spawn_item_own` callers go through this
one path; `pop_resource` already drew its position offsets from
the level random first, matching vanilla's order.
(b) `spawn_destroyed_contents` was wrong in BOTH versions (3
doubles, y random) — now calls `spawn_item_own(stack, position,
0)`, the 5-arg ctor form (2 draws, y = 0.2 exactly).
(c) Idle branch: removed the `slime_step_on` call from
`tick_move_before_block_effects`' idle arm (it cited the
26.1-only method), and gated the bubble-column push on
`step.did_move` at both item call sites (the test `tick()` and
`tick_item_with_block_contacts`) — `BubbleColumnBlock.entityInside`
runs inside `applyEffectsFromBlocks`, which 1.21.11 calls only
on move ticks, so a grounded item in a column lifts on the next
wake-up instead of every tick. `ItemStep.did_move` is now `pub`
(same pattern as `vertical_contact`/`fall_move`). The #4558 rise
test's assertion stands under the gate (16 ticks from ticks=10,
id=1: tick 1 wakes and lifts, tick 5's move clears the ground).

Verified matching, no edits: merge pacing (`moved? 2: 40`,
`tickCount % rate == 0`), escape speed from the item's own
random, `tick_settle` move arm (slime stepOn, friction 0.98 x
block friction, y x -0.5 bounce, needsSync > 0.01, age++ unless
i16::MIN, despawn age >= 6000), fluid heights (> 0.1f32,
0.99 water / 0.95 lava), powder-snow walk check (`PowderSnowSubject`
tag + leather boots; the tag is checked by entity type id, the
1.21.11 form), map-frame ticking (`lan_map_updates` drives
`tick_framed` from the game's item-frame list — observably
equivalent to 1.21.11's `isFramed()`-driven path).

Version-neutral notes: `tick_settle`'s StepOn/swept_inside pass
still runs every tick, but the `block_contact` events dispatch
only to content mods' blocks (`MOD_BLOCKS` gate) — gating them
is a mod-API timing change, out of scope.
`bubble_column_surfaces` (surface-bubble particles) still runs
unconditionally — cosmetic only. No Rust mirror of the orb
renderer anywhere, nor of the entity-representation mechanism
(`onSyncedDataUpdated` / `setEntityRepresentation` /
`ItemStack.isFramed()` — item 35/39 carry-forward). Report gap:
`MapItemSavedData` is in NO report item — the `isFramed()`
holder-retention logic is unassigned. Referral: the
`CauldronInteractions` citations in cauldron.rs,
game/cauldrons.rs, game/remote.rs, game.rs belong to the
cauldron item.

Verification: `cargo test -p horizon-client-lib --lib
--no-default-features entities::` 78 passed, 0 failed, 1
ignored; `entities::destroyed_bundle_environment_tests::` 1
passed (the bundle spill, under the new 5-arg ctor motion);
`cargo test -p horizon-client --lib --no-default-features
game::item_entities::` compiles clean, 0 passed, 1 ignored
(the escape test needs prepared resources). rustfmt --check
clean on game/item_entities.rs and game/entity.rs (leaf files;
entities.rs is a module root, no rustfmt). The #4558 rise test
and the bubble-column surface test early-return locally (no
`.local/horizon-client` resources) — they run for real on CI.
Rank 2 now 260/492 (82 changed, 178 no-change).
work-items.json rank2.closed = 260 on 2026-11-20.

## Item 82 — mobs/monster_brain.rs (rank 2, NO-CHANGE + citations)

6 classes, 27 methods, 5 named: DoNothing, BreezeAi,
BreezeAi$SlideToTargetSink, Shoot, Slide, CreakingAi. The delta
is the 26.x `ActivityData`/`getActivities` API vs the 1.21.11
`makeBrain`/`brainProvider` API, with identical behaviour
content: the literal comparison over the decompile finds ZERO
string or numeric differences in all five decompiled classes,
and Shoot/Slide/SlideToTargetSink have ZERO javap hunks
(bytecode-identical, report noise cf. item 73).

Java delta, jar-verified: 26.1's `getActivities` returns
`List<ActivityData>` built with `ActivityData.create`; 1.21.11's
`makeBrain` takes the Brain and calls `brain.addActivity` /
`addActivityWithConditions`, then `setCoreActivities(Set.of(CORE))`,
`setDefaultActivity(FIGHT)` (breeze) / `setDefaultActivity(IDLE)`
(creaking), `useDefaultActivity()`, plus static SENSOR_TYPES +
MEMORY_TYPES lists and (CreakingAi) `brainProvider()`. The
behaviour tables are identical: breeze core Swim(0.8) +
LookAtTargetSink(45, 90); idle StartAttacking(NEAREST_ATTACKABLE)
0, StartAttacking(HURT_BY) 1, SlideToTargetSink(20, 40) 2,
RunOne(DoNothing(20,100) w1, RandomStroll.stroll(0.6) w2) 3;
fight StopAttackingIfTargetInvalid(negated
wasEntityAttackableLastNTicks(100)) 0, Shoot 1, LongJump 2,
ShootWhenStuck 3, Slide 4, conditions ATTACK_TARGET present +
WALK_TARGET absent. Creaking core Swim-subclass(0.8) +
LookAtTargetSink(45, 90) + MoveToTargetSink; idle
StartAttacking 10, SetEntityLookTargetSometimes(8.0, 30-60) 11,
RunOne(stroll 0.3 w2, walk-from-look 0.3/3 w2, DoNothing(30,60)
w1) 12; fight walk-from-attack-target(1.0) 10, MeleeAttack(40)
11, StopAttackingIfTargetInvalid 12, condition ATTACK_TARGET
present. Small behaviour-neutral differences: the breeze
hurt-by finder calls `getHurtBy()` in 1.21.11 vs inlining its
body in 26.1 (`getHurtBy` exists in both jars with that body);
the creaking idle condition drops a dead `instanceof Creaking`
check; 26.1's DoNothing overrides `getRequiredMemories()`
returning the empty set, 1.21.11 inherits the empty default.
`isAttackTargetStillReachable` and both `updateActivity`
methods are identical. Breeze sensors (4) and memories (17)
and creaking sensors (2) and memories (11) are listed in the
triage note. CreakingAi$1 (the anonymous Swim subclass) adds a
`canMove()` gate to the swim start condition, in both versions.

Verified matching, no edits: the Rust's sensor lists (breeze
4, creaking 2, in 1.21.11 order, first scans delayed by
`next_int_bound(20)` mob-random draws), the shared `Memories`
struct (all 17 breeze + all 11 creaking memories), both
activity tables (every behaviour, priority and parameter),
the default activities (breeze Fight, creaking Idle),
`update_activity` (frozen creaking -> default; else FIGHT iff
the species' fight conditions — exactly
`setActiveActivityToFirstValid([FIGHT, IDLE])`), and the Swim
gate (`*creaking && !can_move`).

Fixes (comment-only): `breeze_packages` / `creaking_packages`
docs cite `makeBrain` instead of `getActivities`. Referrals:
`warden_brain.rs:80,277` still cites `getActivities` (the
warden's item); the shared behaviour implementations are their
own items' scope.

Verification: `cargo test -p horizon-client-lib --lib
--no-default-features mobs::monster_brain::` 0 tests (none live
in the module); adjacent `mobs::breeze::` 7 passed,
`mobs::creaking::` 5 passed. rustfmt --check clean on
monster_brain.rs. Rank 2 now 261/492 (82 changed, 179
no-change). work-items.json rank2.closed = 261 on 2026-11-20.

## Item 83 — loot.rs (rank 2, CHANGED)

15 classes, 67 methods, 1 named: EntityPredicate,
LootPoolEntryContainer (+$Builder), SetBannerPatternFunction
(+$Builder), SetBookCoverFunction, SetCustomDataFunction,
SetCustomModelDataFunction, SetInstrumentFunction,
SetLoreFunction (+$Builder), SetWrittenBookPagesFunction,
ToggleTooltips, TimeCheck (+$Builder). The literal comparison
finds one string delta — TimeCheck's `"clock"` (the removed
codec field) — and no real numeric delta.

Java delta, jar-verified: (1) TimeCheck loses the 26.x
WorldClock: 26.1's record is `(Holder<WorldClock> clock,
period, value)` and `test` reads
`level.clockManager().getTotalTicks(clock)`; 1.21.11's is
`(period, value)` and `test` reads `level.getDayTime()`, then
`% period` and `value.test(ctx, (int) t)`. (2)
SetInstrumentFunction: 26.1's `options` is a
`HolderSet<Instrument>` (`RegistryCodecs.homogeneousList` —
tag or explicit list) picked with
`options.getRandomElement(random)`; 1.21.11's is a
`TagKey<Instrument>` (`TagKey.hashedCodec`) picked with
`registry.getRandomElementOf(options, random)`. The only
vanilla 1.21.11 table using it is chests/pillager_outpost.json:
`"options": "#minecraft:regular_goat_horns"`. (3) API shape
only: `MAP_CODEC` -> `CODEC` + `codec()` -> `getType()` on all
ten function/condition/entry classes (all `run()` bodies
identical); LootPoolEntryContainer drops `Validatable` (its
`validate` inlines the per-condition loop);
SetCustomModelDataFunction and TimeCheck swap `validate` for
`getReferencedContextParams`; SetLoreFunction swaps
`DataFixUtils.orElse` for `orElseGet`; EntityPredicate swaps
`Entity.typeHolder()` for `Entity.getType()`. The two $Builder
classes have zero javap hunks (report noise).

Verified matching, no edits: the Rust's `set_instrument`
already handles the 1.21.11 tag form — `holder_set` resolves
`"#tag"` via `tag()` (nested tags expanded in place, tag order,
duplicates dropped), and `LootData::load` loads `instrument`
tags; the pick is `next_int_bound(len)`, matching
`getRandomElementOf`. All eight function arms exist in `run()`.
`entity_matches` matches the type by id string (or `#tag`/list).
loot.rs has no codec/validate mirrors, so the rename delta
needs no change. No vanilla 1.21.11 table uses `time_check`
or `weather_check` (grep over the client.jar data pack).

Fixes: the `set_instrument` comment now cites
`Registry.getRandomElementOf` (the 1.21.11 form), and a new
test `set_instrument_picks_from_its_tag` pins the 1.21.11 JSON
form (tag file with the vanilla four; 80 seeds see all four).
The existing list-form test is kept (the Rust's `holder_set`
stays lenient). This discharges item 29's gap note for the
loot path: in 1.21.11 there is no EitherHolder here — the
options are always a tag.

Version-neutral gap: `minecraft:time_check` (and
`weather_check`) has NO arm in either loot engine — unknown
conditions fail, so a data pack using `time_check` would
silently get no drops. Unobservable in vanilla data; adding it
needs day-time plumbing into `Context`/`roll`/`fill` (the
`NativeLootLevel` already carries `day_time` for mods' kinds).
Item 50's carry-forward stands (five missing block-only kinds,
`bonus_rolls`, `choose` zero-weight).

Verification: `cargo test -p horizon-world --lib
--no-default-features loot::` 34 passed; `set_instrument` 2
passed (list + tag forms). rustfmt --check clean on loot.rs
(module root — declares `mod odds` — check only). Rank 2 now
262/492 (83 changed, 179 no-change). work-items.json
rank2.closed = 262 on 2026-11-20.

## Item 84 — arrows.rs (rank 2, NO-CHANGE)

4 classes, 12 methods, 6 named: EnderMan, AbstractArrow,
ThrownTrident, CrossbowItem. The entire delta is API shape +
visibility: the literal comparison finds ZERO string or
numeric differences in all four classes.

Java delta, jar-verified: AbstractArrow.onHitEntity and
ThrownTrident.onHitEntity swap `entity.is(ENDERMAN)` (26.1)
for `entity.getType() == ENDERMAN` (1.21.11) — same result;
the rest of the 199-line AbstractArrow hunk is bytecode-offset
renumbering (the body is identical: ignite 5s except endermen,
hurtOrSimulate, enderman early-return, arrow count +1,
doKnockback, post-attack/post-hurt effects, PLAY_ARROW_HIT_SOUND,
piercedAndKilledEntities + KILLED_BY_ARROW, pitch
`1.2 / (0.9 + random * 0.2)`, pierce > 0 flies on else discard;
on a failed hurt: restore fire ticks, deflect(REVERSE),
velocity x 0.2, lengthSqr < 1e-7 + pickup ALLOWED -> drop at
0.1f + discard). `isAttackable` swaps `is(tag)` for
`getType().is(tag)` — same result. CrossbowItem is pure
renames, verified bytecode-identical from both jars:
`ChargedProjectiles.ofNonEmpty` -> `of`,
`ChargedProjectiles.itemCopies` -> `getItems`, `ItemStack.is(Object)`
-> `is(Item)` (x3). EnderMan: two methods private ->
package-private, `Potions.WATER` `Holder$Reference` -> `Holder`.
ProjectileDeflection.REVERSE (verified from the 1.21.11 jar):
rotation 170 + random x 20, velocity x -0.5, yaw/yRotO +=
rotation — folded with onHitEntity's 0.2 (arrow: x -0.1) and
the trident's (0.02, 0.2, 0.02) (x (-0.01, -0.1, -0.01)).

Verified matching, no edits: `finish_entity_hit` (Hurt ->
sound + pierce == 0 discards; Refused -> turn back; trident
Hurt/Refused -> bounce; Deflected -> pierced.pop()),
`deflect_reverse` (x -0.1, yaw += 170 + random x 20, lengthSqr
< 1e-7 + ALLOWED -> drop + removed), `trident_bounce`
(dealt_damage, TRIDENT_HIT (1.0, 1.0), x (-0.01, -0.1, -0.01)),
`projectile_hit_inner` (fire-tick save/restore, ignite 5s,
enderman dodge -> Dodged, Punch knockback `* 0.6 *
(1 - resistance)` with y 0.1, post-attack/post-hurt effects,
stuck arrow for a hurt player, KILLED_BY_ARROW for a
player-shot kill). No stale citations: the crossbow's charged
projectiles are a plain vec (no `ofNonEmpty`/`itemCopies`
anywhere).

Version-neutral gaps: no enderman early-return on a LANDED
hurt (teleport failed) — vanilla skips the ignite, knockback,
post-attack/post-hurt effects, arrow count, hit sound and
KILLED_BY_ARROW, and the arrow flies through / the trident
doesn't bounce; the Rust applies them and spends/bounces the
projectile. Only reachable when the teleport fails (the common
dodge path matches). Mob arrow count is never incremented on a
hit (the decay rule is unverified, so no fix). `deflect_reverse`
/ `trident_bounce` don't update yRotO (render only).

Verification: `cargo test -p horizon-client-lib --lib
--no-default-features arrows::` 7 passed, 1 ignored. No files
edited. Rank 2 now 263/492 (83 changed, 180 no-change).
work-items.json rank2.closed = 263 on 2026-11-20.

## Item 85 — game/item_draw.rs (rank 2, CHANGED)

2 classes, 32 methods, 4 named: ItemEntityRenderer,
BundleContents. The version delta is API shape only (the
literal comparison finds ZERO differences in both classes),
but the file's `BundleContents.weight` mirror was wrong in
both versions and is fixed here.

Java delta, jar-verified: ItemEntityRenderer is a package
move only (`state.level.CameraRenderState` -> `state.
CameraRenderState`), zero body change — hover `-minY +
0.0625`, bob `sin(age/10 + bobOffset) * 0.1 + 0.1`, spin
`age/20 + bobOffset`, `submitMultipleFromCount` with the
cluster seed (3D: copies at +/-0.15 on three axes; flat:
z-step = zsize x 1.5, jitter +/-0.075 x/y). BundleContents:
26.1 holds `List<ItemStackTemplate>` + a lazily memoized
`Supplier<DataResult<Fraction>>` weight; 1.21.11 holds
`List<ItemStack>` + an eagerly computed `Fraction` field.
Same values: nested bundle = 1/16 + inner weight
(`BUNDLE_IN_BUNDLE_WEIGHT = 1/16` in both), beehive with
bees = 1 (`BEEHIVE_WEIGHT = success(ONE)` in 26.1), default
= 1 / maxStackSize; content weight = sum of `getWeight x
count`; the codec gains `flatXmap` + `checkAndCreate`
("Excessive total bundle weight" on fraction overflow).
Renames: `getSelectedItemIndex()` -> `getSelectedItem()`
(the index), 26.1's `getSelectedItem()` -> `hasSelectedItem()`;
1.21.11-only `getItemUnsafe`, `itemsCopy`. `equals`/`hashCode`
are count-insensitive in 1.21.11 (`ItemStack.listMatches` /
`hashStackList`) vs count-sensitive in 26.1 — no Rust mirror.

Verified matching: `cluster_poses_with_depth` matches
`submitMultipleFromCount` exactly; `horizon-items/bundle.rs`
already implements the full weight model exactly (nested
1/16, beehive 1, exact reduced `Fraction`).

Fix: `bundle_fullness` (the `NumericProperty::BundleFullness`
host service) summed `count / max_stack_size` — a nested
bundle weighed count/1 = 1.0 (vanilla: 1/16 + inner) and a
beehive with bees weighed 1/64 (vanilla: 1). It now uses the
full `getWeight` model via a new `bundle_item_weight` helper
over the JSON form (recursive nested bundles, non-empty
`minecraft:bees` -> 1, else count/max). New test
`bundle_fullness_weighs_nested_bundles_and_beehives` pins
0.5 / 1.0 / 1/64 / 1/16+0.25 / 1/16.

Version-neutral note: the `!`-patch removal form
(`!minecraft:bees`) is not consulted by the new helper; the
resolved components map makes it unobservable here.

Verification: `cargo test -p horizon-client --lib
--no-default-features game::item_draw::` 4 passed, 0 failed.
rustfmt clean on game/item_draw.rs (leaf file). Rank 2 now
264/492 (84 changed, 180 no-change). work-items.json
rank2.closed = 264 on 2026-11-20.

## Item 86 — game/mob_blocks.rs (rank 2, CHANGED)

16 classes, 32 methods, 4 named: CopperGolemModel,
ElderGuardianParticle(Group), CopperGolemRenderer,
BlockDecorationLayer, CarriedBlockLayer, IronGolemFlowerLayer,
MushroomCowMushroomLayer, SnowGolemHeadLayer, plus 5
missing-side in 1.21.11 (`BlockModelRenderState`,
`BuiltInBlockModels` + inners — the 26.x block-model
render-state system, absent in 1.21.11).

Java delta, jar-verified: `CopperGolemModel.
applyBlockOnAntennaTransform` translates (0, -1.75, 0) in 26.1
vs (0, -2.25, 0) in 1.21.11. BlockDecorationLayer drops the
fixed `UNIT_CUBE_BOTTOM_CENTER_TO_ANTENNA_CENTER` matrix and
branches by block kind: not a statue -> rot Z 180; a statue,
skull, banner or chest -> rot Y 180; flower bed -> translate
(-0.25, -1.5, -0.25); else not a statue -> (-0.5, -1.5, -0.5);
a statue -> (-0.5, 0, -0.5). The ordinary-block and flower-bed
nets are IDENTICAL in both versions (hand-verified: both send
(x,y,z) to (0.5-x, -y-0.75, z-0.5) / (0.25-x, -y-0.75,
z-0.25) — the -0.5 moved from the layer into the model). The
real differences are statues ((0.5-x, y-2.25, 0.5-z): half a
block lower, Y-flipped instead of Z-flipped) and skulls/
banners/chests (extra Y-180, net R_x180: (x-0.5, -y-0.75,
0.5-z)). CopperGolemRenderer stores `Optional<BlockState>`
instead of resolving into a BlockModelRenderState. The other
layers keep identical transforms and gates (carried block,
poppy on `offerFlowerTick != 0`, mooshroom variant mushrooms,
carved pumpkin on `hasPumpkin`); only the submit mechanism
changes (`submitBlock`/`submitBlockModel` vs the pre-resolved
render state). ElderGuardianParticle(Group): visibility +
package moves only.

Verified matching: `carried_block_matrix`,
`golem_flower_geometry` (counter gate, poppy),
`worn_block_geometry` (sheared/baby gates, all placements),
`antenna_block_geometry`'s property-by-property
`BlockItemStateProperties.apply`.

Fix: `antenna_block_matrix(head, flowerbed_shift)` ->
`antenna_block_matrix(head, block: &str)` with the 1.21.11
form (model -2.25; Z-180 unless a statue; Y-180 for statues
(`copper_golem_statue` suffix), skulls (`skull_kind`),
banners (`_banner` suffix), chests (chest/trapped/ender);
the three translates). Stale 26.x citations fixed
(`BlockModelRenderState`/`UNIT_CUBE`/`BuiltInBlockModels` —
none remain in the tree). The antenna test is rewritten per
block kind (stone, wildflowers, copper_golem_statue,
skeleton_skull) with hand-computed points.

Version-neutral note: skulls/banners/chests on the antenna
draw through the baked-model path, not their block-entity
renderers (vanilla's `submitBlock` routes them to the BERs) —
pre-existing, out of scope.

Verification: literal comparison — CopperGolemModel only
1.75/2.25, BlockDecorationLayer only the new 1.21.11
constants, IronGolemFlowerLayer only the `0` gate, the rest
zero. `cargo test -p horizon-client --lib --no-default-features
game::mob_blocks::` 3 passed. rustfmt clean on
game/mob_blocks.rs (leaf file). Rank 2 now 265/492
(85 changed, 180 no-change). work-items.json rank2.closed =
265 on 2026-11-20.

## Item 87 — client/gui/components (rank 2, CHANGED)

28 classes, 72 methods, 0 named in Horizon (widely named,
~57 files): the selection-list/chat/button/cycle-button/
text-widget/scrollable-layout/string-widget inner classes,
plus 2 missing-side in 1.21.11 (`AbstractScrollArea$
ScrollbarSettings`, `ChatComponent$DisplayMode`).

Java delta, jar-verified: the 26.x render-state extraction
API is gone (`extractContent`/`extractContents`/
`extractWidgetRenderState`/`extractRenderState`/
`extractBackground` -> `render*`, `GuiGraphicsExtractor` ->
`GuiGraphics`, `GuiMessage`/`GuiMessageTag` move from
`multiplayer.chat` to `client`); visibility + requireNonNull
noise. The ScrollbarSettings migration is the substance:
26.1's settings record (width 6, min height 32, scrollRate
int) is gone; 1.21.11's `AbstractScrollArea.scrollRate()` is
abstract, overridden per class as a double —
FittingMultiLineTextWidget 9.0, ScrollableLayout$Container
10.0, `MultiLineEditBox` 9.0/2.0 = 4.5 (26.1 truncated it to
an int, 4 — a real change), `AbstractSelectionList`
defaultEntryHeight/2.0. ScrollableLayout: 26.1's
ReserveStrategy (BOTH, hardcoded) + scrollbarSpacing 4 +
minHeight vs 1.21.11's fixed `SCROLLBAR_SPACING = 4` /
`SCROLLBAR_RESERVE = 10`; widths identical (max(content+20,
min), content at +10), height min(content, max) vs
clamp(container, min, max); `ensureVisible` 14.0 (1.21.11)
vs scrollRate 10.0 (26.1); `getBorderForArrowNavigation`
returns the full rect (1.21.11) vs delegating to the focused
child (26.1). `ChatComponent$DisplayMode` + the restricted
prompt ("chat_screen.restricted") are 26.1-only — the Rust
has no mirror, matching 1.21.11 by omission.
`ComponentRenderUtils.clipText` moved to `StringWidget.
clipText` (same body); `ComponentRenderUtils` keeps only
`wrapComponents` in 1.21.11.

Verified matching: dialog_screen (10 each side, min(content,
max)), server_links (reserve 10, wheel x10), realms PdpList
wheel 22 = 44/2, `screen_draw::clip` (= `StringWidget.
clipText`). No `ensureVisible`, FittingMultiLineTextWidget or
DisplayMode mirrors exist.

Fix: `multiline_edit.rs` SCROLL_RATE 4.0 -> 4.5
(`MultiLineEditBox.scrollRate`, 9.0/2.0 — MultiLineEditBox
is item 529's class; the fix is this item's migration,
jar-verified both sides). Citations: `screen_draw.rs`
clipText -> `StringWidget.clipText`; `chat.rs` "dropped" ->
"moved to `StringWidget.clipText`"; `dialog_screen.rs` /
`server_links.rs` ReserveStrategy.BOTH -> `SCROLLBAR_RESERVE`;
`realms.rs` defaultSettings -> `AbstractSelectionList.
scrollRate`.

Referrals: AbstractSelectionList -> item 13 (scrollbarWidth
inlined as 6); ChatComponent/ComponentRenderUtils -> item 17;
DebugScreenOverlay -> item 519 (the "Debug charts:" line
joins the network chart onto one line in 1.21.11);
AbstractScrollArea -> item 528; AbstractTextAreaWidget/
MultiLineEditBox -> item 529 (verify the 4.5 fix). The
`GuiGraphicsExtractor` name is cited tree-wide — each
screen/widget item fixes its own citations.

Verification: literal comparison — 9 classes zero;
AbstractSelectionList only the inlined 6; ChatComponent
only the restricted strings; DebugScreenOverlay only the
chart-line splits; FittingMultiLineTextWidget 9 vs 9.0;
ScrollableLayout 10.0/14.0/2/20. `cargo test -p
horizon-client-render --lib --no-default-features
multiline_edit::` 8 passed. rustfmt clean on the five leaf
files; chat.rs not reformatted (module root, pre-existing
drift at 1401 + chat_tests.rs 514). Rank 2 now 266/492
(86 changed, 180 no-change). work-items.json rank2.closed =
266 on 2026-11-20.

## CI proof — 266/492 milestone (dispatched 2026-11-20 23:48 NPT)

Run 44789092726 (`gh workflow run port-verify.yml -f version=1.21.11
--ref behaviour/1.21.11`) on 0f75c53 (item 87, rank 2 266/492).
In progress at dispatch; the previous run on this branch
(44538822133, Nov 19) took 1h31m. Record the profile into
pins/1.21.11/ when it lands; do not merge on it (rank 2 is
not done).

## Item 88 — game/end.rs (rank 2, NO-CHANGE)

5 classes, 9 methods, 6 named: ThrownEnderpearl, EnderEyeItem,
TheEndGatewayBlockEntity, EndPlatformFeature, EndPodiumFeature.

Java delta, jar-verified: the pearl's `onHit` has two teleport
branches. The player branch (teleport, resetFallDistance,
resetCurrentImpulseContext, 5 damage, sound) is identical in
both versions. The non-player-owner branch gains, in 26.1 only,
`if (newOwner instanceof LivingEntity)
livingEntity.resetCurrentImpulseContext()` — 1.21.11 has no such
call, because the impulse context lives on `Player` there (it
moved up to `LivingEntity` in 26.x, with a Vec3-taking setter).
Everything else is API shape: `BlockState.is(Object)` ->
`is(Block)` in all five classes (END_GATEWAY, END_PORTAL_FRAME
x2, BEDROCK, END_STONE, the feature material checks), and
`Level.getRandom()` -> the `Level.random` field in EnderEyeItem.

Verified matching: the Rust's player-pearl path (creatures.rs)
is the 1.21.11 player branch exactly (5% endermite, stop_riding,
velocity ZERO, fall distance 0, impulse reset, 5 damage, sound);
the `ender_pearl_clears_impulse_before_next_fall` test pins the
player branch, which is the same in both versions. The
non-player branch has no Rust mirror (only player-owned pearls
teleport; mobs have no impulse context), so the 26.1-only reset
is unobservable — and unreachable in vanilla play (only players
throw pearls; dispenser pearls have no owner). Gateway
(end.rs + end_gateway.rs), platform (`createEndPlatform`),
podium (`worldgen/end_podium`) and eye mirrors all audited;
no stale citations.

Note: `javap_diff.py` renumbers bytecode offsets to sequential
instruction indices — do not compare its offsets to raw javap
offsets (the pearl's LivingEntity reset is diff-offset 178,
raw offset 384).

Verification: literal comparison — all five classes zero string
and zero numeric differences. `cargo test -p horizon-client
--lib --no-default-features game::end::` 6 passed, 2 ignored
(fixtures). No files edited. Rank 2 now 267/492 (86 changed,
181 no-change). work-items.json rank2.closed = 267 on
2026-11-20.

## Item 89 — level/mod.rs (rank 2, NO-CHANGE + citations)

1 class, 19 methods, 5 named: `net.minecraft.world.level.Level`.

Java delta, jar-verified: the 26.x WorldClock is gone —
`getOverworldClockTime`, `getDefaultClockTime`,
`getClockTimeTicks`, the abstract `clockManager`, and the
crash-report "Level time" detail (`"%d game time, %d day time"`)
are all 26.1-only; 1.21.11 has plain `getDayTime()` reading
`levelData.getDayTime()` (the same removal items 44/74/83
ported). `prepareWeather()` is 1.21.11-only: a saved rain or
thunderstorm opens at full strength (rainLevel/thunderLevel
1.0). Renames only: `random` protected -> public;
`soundSeedGenerator` -> `threadSafeRandom` (both
`createThreadSafe()`, both feed `playSeededSound` with
`nextLong()`); ctor `randValue` from
`createThreadLocalInstance()` (26.1) vs `create()` (1.21.11)
— both unseeded draws; `isInsideBuildHeight` ->
`isOutsideBuildHeight` (inverted at both call sites);
`ChunkPos.pack` -> `asLong`; `is(Object)` -> `is(Block)` x2
(COMPARATOR).

Verified matching: no WorldClock mirrors in level/mod.rs;
`getDayTime` already the plain read (item 44); `prepareWeather`
already mirrored in save.rs (raining -> (1.0, 1.0),
thundering -> (1.0, 1.0)) and world_loading.rs (the --weather
override); `hub.next_sound_seed()` mirrors the seeded-sound
random; `level_loading.rs` already cites
`isOutsideBuildHeight`; the Rust's `ChunkPos::pack()` is a
Rust-side API, not a Java citation.

Citation fixes: level/mod.rs "overworld clock" -> "day time"
in the `getDayTime` comment; frames.rs `isInsideBuildHeight`
-> `isOutsideBuildHeight` (verified from the 1.21.11 jar:
`ItemFrameItem.mayPlace` fails when outside); ticks.rs
"thread-local random" -> "unseeded random,
`RandomSource.create()`"; save.rs gains the `prepareWeather`
citation above the weather init.

Verification: literal comparison — only 26.1-only strings are
`"%d game time, %d day time"` + `"Level time"`, only 26.1-only
number `0L`. `cargo check -p horizon-world --lib` and `-p
horizon-client --lib` clean; ticks tests 0 (no test module).
All edits comment-only. Rank 2 now 268/492 (86 changed, 182
no-change). work-items.json rank2.closed = 268 on 2026-11-20.

## Item 90 — game/jukebox.rs (rank 2, CHANGED)

2 classes, 18 methods, 5 named: JukeboxPlayable,
JukeboxBlockEntity.

Java delta, jar-verified: the EitherHolder migration (same
pattern as items 29/35) — 26.1's `JukeboxPlayable.song` is a
resolved `Holder<JukeboxSong>`; 1.21.11's is an
`EitherHolder<JukeboxSong>` (key or inline value), unwrapped
against a `HolderLookup.Provider`; `JukeboxSong.fromStack`
takes the provider in 1.21.11 (all four call sites:
getComparatorOutput, setTheItem, tryForcePlaySong,
loadAdditional). `addToTooltip` shows the gray description
only when the unwrap succeeds — an unknown song id shows NO
line. The rest is API shape: `getRandom()` -> the `random`
field (note offset 0.7f), `is(Object)` -> `is(Block)`.

Verified matching: `jukebox_playable` already implements the
EitherHolder (String key / inline / removed / item default);
`start_song` and `tick_jukebox` do nothing for an unknown id;
`jukebox_signals` gives the song's comparator output, 0 when
unresolved; insertion/eject/level-event paths match.

Fix: `item_tooltip.rs::jukebox_line` fell back to the raw
`jukebox_song.<id>` key when the description key was
untranslated, so a disc naming an unknown song showed the raw
key. The fallback is dropped — an untranslated key hides the
line (1.21.11's unwrap-failure behaviour; every vanilla song's
description IS its `jukebox_song.*` key, so the lang table
doubles as the resolution check at this layer). New test
`a_disc_naming_an_unknown_song_shows_no_jukebox_line`.

Version-neutral gaps: an inline `jukebox_playable` definition
does not play (the disc goes in, `playing` is set, but
`start_song` is never called — `song_of_record` returns None
for inline songs); a datapack song whose description is a
literal shows no tooltip line. Both wrong in 26.1 too; the
inline-song plumbing is out of scope here.

Verification: literal comparison — both classes zero.
`cargo test -p horizon-client-lib --lib --no-default-features
item_tooltip::` 23 passed, 1 ignored (item_tooltip is declared
in client_lib.rs, so its tests live in the horizon-client-lib
binary). rustfmt clean on item_tooltip.rs (leaf). Rank 2 now
269/492 (87 changed, 182 no-change). work-items.json
rank2.closed = 269 on 2026-11-21.

## Item 91 — network/codec (rank 2, NO-CHANGE)

34 classes, 68 methods, 0 named: `ByteBufCodecs$1`–`$34`
(the anonymous StreamCodec implementations).

Java delta: none. `javap_diff.py 91` is +0/-0 across all 34
inner classes; the outer `ByteBufCodecs` (in NO report item)
was compared directly with the same normalizer and is also
identical; the literal comparison is zero too. The report's
"68 changed methods" are pure decompile noise — the same
failure mode as item 73.

Verified matching: the Rust cites `ByteBufCodecs.registry`,
`.holder`, `.holderSet`, `.readCount`, all identical in both
versions; no citation is stale.

Verification: `cargo test -p horizon-protocol --lib buf::`
9 passed. No files edited. Rank 2 now 270/492 (87 changed,
183 no-change). work-items.json rank2.closed = 270 on
2026-11-21.

## Item 92 — streaming.rs (rank 2, NO-CHANGE + citations)

9 classes, 36 methods, 3 named: CompiledSectionMesh,
SectionRenderDispatcher$RenderSection$CompileTask,
ChunkGenerationTask, ChunkTaskDispatcher,
ChunkTaskPriorityQueue, ChunkTracker, ChunkTrackingView (+
$Positioned), IOWorker$PendingStore.

Java delta, jar-verified: CompiledSectionMesh is the 26.x
render-state rework reversed — 26.1's `draws: Map<Layer,
SectionDraw>` + uploaded-flag AtomicBoolean maps vs 1.21.11's
`buffers: Map<Layer, SectionBuffers>` with `uploadMeshLayer`
(create/reuse vertex + index GPU buffers, set index count +
type) and `uploadLayerIndexBuffer` (the translucency re-sort).
`CompileTask.doTask` returns `SectionTaskResult` directly in
26.1 vs `CompletableFuture<SectionTaskResult>` in 1.21.11.
The server classes are API shape only: `ChunkPos.x()`/`z()` ->
`.x`/`.z` fields, `pack` -> `asLong`/`toLong`, `unpack(long)`
-> `new ChunkPos(long)`, visibility private -> package-private;
the ChunkTracker 3x3 loops and ChunkGenerationTask radii are
identical logic.

Verified matching: no Rust mirror of the SectionDraw/
SectionBuffers divide (the section pipeline is the Rust's own
wgpu design); the streaming worker's citations (IOWorker,
`doTask`, `ChunkTaskPriorityQueue.release`, ChunkGenerationTask
x-outer/z-inner order, `ChunkTrackingView.isInViewDistance`,
`ChunkTracker.computeLevelFromNeighbor`) all name members that
exist identically in both versions.

Citation fixes: `streaming.rs` cited
`CompiledSectionMesh.compileTaskStartTimeNs` /
`getCompileTaskStartTime`, which exist in NEITHER 26.1 nor
1.21.11 (verified by javap on both jars + a package-wide grep)
— 26.3-only members inherited from the Rust's 26.3 base. Both
dropped; the docs now describe the Rust's own `sequence` clock.

Referrals: SectionRenderDispatcher (outer) -> item 509 (rank
3: the 26.1-only buffer-pool sizes, the `pC/aB` vs `pC/pU/aB`
debug stats, the 1.21.11 upload-task profiler names);
`LevelRenderer.extractSectionDrawGroups` (gfx.rs:8686, 8933)
exists in neither version — 26.3-only; LevelRenderer is item
503 (rank 3).

Verification: literal comparison — the six server classes +
IOWorker zero; CompiledSectionMesh only the 1.21.11 buffer
labels + usage flags 40/72. `cargo test -p
horizon-client-render --lib --no-default-features streaming::`
29 passed, 2 ignored. Rank 2 now 271/492 (87 changed, 184
no-change). work-items.json rank2.closed = 271 on 2026-11-21.

## Item 93 — mob_ai_probe.rs (rank 2, CHANGED)

4 classes, 16 methods, 5 named: TradeWithVillager, Horse,
DoorBlock, BeehiveBlockEntity.

Java delta, jar-verified — all of it in Horse: (1) the 1.21.11
ctor sets DANGER_OTHER and DAMAGE_OTHER malus to -1.0 (26.1
does not; AbstractHorse sets none in either version; among the
equines only Horse and ZombieHorse set them in 1.21.11). (2)
The six `HORSE_*_BABY` sounds are 26.1-only — 1.21.11 plays the
adult sounds at every age (gallop breathe keeps its 1-in-10
roll and volume x0.6). (3) 26.1's mobInteract shortcuts to
super when a baby is offered a GOLDEN_DANDELION (a 26.x-new
item); 1.21.11 has no such branch. (4) BABY_DIMENSIONS is a
`<clinit>`-only change: 26.1 scales 0.7 with the PASSENGER
attachment at height - 0.125; 1.21.11 scales 0.5 with the
attachment at height + 0.125 — the same 0.7 -> 0.5 for
SkeletonHorse and ZombieHorse (Llama 0.5 in both; Donkey/Mule
define none). TradeWithVillager is byte-identical; DoorBlock
is `is(Object)` -> `is(Block)` x3; BeehiveBlockEntity is
visibility + `getRandom()` -> the field (the release logic is
the same body with shifted locals).

Verified matching: no baby-sound or golden-dandelion mirror in
the Rust (already 1.21.11); the baby PASSENGER attachment has
no Rust mirror and is unobservable (a foal cannot be ridden).

Fixes: `mobs/ai.rs` gains a Horse/ZombieHorse malus arm
(DangerOther -1.0, DamageOther -1.0); `mobs/species.rs` moves
Horse, SkeletonHorse and ZombieHorse to `Baby::Scale(0.5,
None)`. New equine tests pin the baby scales (with Donkey/Mule
0.5 controls) and the malus (donkey keeps DangerOther 8.0).

Verification: literal comparison — the other three classes
zero; Horse only 0.7F vs -1.0F/0.5F. `cargo test -p
horizon-client-lib --lib --no-default-features mobs::equine::`
12 passed, 1 ignored. rustfmt clean on species.rs/ai.rs;
equine.rs only the pre-existing drift at 630. Rank 2 now
272/492 (88 changed, 184 no-change). work-items.json
rank2.closed = 272 on 2026-11-21.

## Item 94 — game/maps.rs (rank 2, CHANGED)

5 classes, 16 methods, 5 named: CompassItem, MapIndex,
MapItemSavedData (+ HoldingPlayer), ExplorationMapFunction.

Java delta, jar-verified: the map save paths are the 26.x
SavedDataType rework reversed — 26.1's MapIndex TYPE is
`maps/last_id` (file `data/minecraft/maps/last_id.dat`) vs
1.21.11's plain `"idcounts"` (`data/idcounts.dat`); `MapId.key()`
is `"maps/" + id` in 26.1 vs `"map_" + id` in 1.21.11 (verified
from the `makeConcatWithConstants` bootstrap args — MapId is in
no report item, and the javap normalizer hides string constants).
The NBT shape is the same (`{"map": int}`, default -1).
`tickCarriedBy` loses its ItemFrame parameter in 1.21.11 — the
stack itself knows (`ItemStack.isFramed()` / `getFrame()`); the
holder-retention, no-player-markers-while-framed and frame-marker
outcomes are identical. `isTrackedCountOverLimit` is `>` in 26.1
vs `>=` in 1.21.11 — at exactly 256 tracked decorations 1.21.11
refuses the 257th banner marker. The rest is API shape:
CompassItem `is(Block)`, ExplorationMapFunction `MAP_CODEC` ->
`CODEC` + `codec()` -> `getType()`, HoldingPlayer visibility.

Verified matching: `tick_framed` (explicit frame param) matches
1.21.11's outcomes (holders kept while framed, no player marker
without a carried copy, stale-frame removal, direction*90, dirty
on change) — the Rust has no entity-representation mirror, so the
explicit form is the right Rust shape; its doc now says so instead
of citing the bogus `placedInFrame`. No `MAP_CODEC`/`codec()`
citation; compass `useOn` compares block names.

Fixes: `tracked_over_limit` `>` -> `>=` (new boundary test:
255 tracked -> not over, 256 -> over). Map paths: `map_path`
builds `data/map_<id>.dat`; `next_map_id` reads and `save_maps`
writes `data/idcounts.dat`; the `map_files` harness op and the
exploration-map test file read the new paths. Before this fix a
1.21.11 world opened by the Rust found no counter file, so the
first new map took id 0 and collided with the world's existing
`map_0.dat`.

Verification: literal comparison — only MapIndex differs
(`maps/last_id` vs `idcounts`). `cargo test -p horizon-client
--lib --no-default-features game::maps::` 8 passed, 6 ignored
(fixtures). rustfmt --check clean on maps.rs (module root, not
reformatted). Rank 2 now 273/492 (89 changed, 184 no-change).
work-items.json rank2.closed = 273 on 2026-11-21.

## Item 95 — level/tick.rs (rank 2, CHANGED)

5 classes, 16 methods, 5 named: BambooStalkBlock, CauldronBlock,
ChorusFlowerBlock, KelpBlock, BarrelBlockEntity.

Java delta, jar-verified: bamboo's support is the 1.21.11
`bamboo_plantable_on` tag (#sand + #dirt + bamboo/sapling +
gravel/suspicious_gravel) vs 26.1's `supports_bamboo` (#sand +
#substrate_overworld + the same rest) — a real set difference,
verified from both jars' tag JSONs. Bamboo bonemeal loses its
26.x guards in 1.21.11: `isValidBonemealTarget` checks only
height < 16 and stage != 1 (no build-height, no empty check);
`performBonemeal` drops the build-height check (the empty check
stays). Chorus flower support is hardcoded END_STONE in 1.21.11
(26.1's tag held only end stone). Kelp `canAttachTo` is
hardcoded not-MAGMA in 1.21.11 (26.1's tag held only magma);
`getStateForPlacement`'s `isFull()` vs `getAmount() == 8` is the
same result for water. CauldronBlock's ctor takes the
`CauldronInteraction.EMPTY` map in 1.21.11 (the table moved
classes in 26.x). BarrelBlockEntity is visibility + the random
field.

Fixes: the `bamboo_plantable_on` tag in the live Tags struct,
the survival registry and the placement check; the bamboo
bonemeal guards dropped in bonemeal.rs (valid + perform);
chorus flower checks end stone in survive.rs, tick.rs (both
randomTick sites) and the registry rule (the dead
`chorus_flower` set + the tags! entry removed); kelp refuses
magma in survive.rs and the registry resolves `no_kelp` from
`block_states("minecraft:magma_block")` (the `cannot_support_kelp`
tags! entry removed). Cauldron citations renamed
`CauldronInteractions` -> `CauldronInteraction` (7 spots,
comment-only; verified 1.21.11 has only the singular class).

Referrals: the cauldron interaction table contents -> item 558
(rank 3; Dispatcher/Interactions/InteractionMap are 683/735/791).
The remaining `supports_*` tags (cactus, sugar cane, cocoa,
azalea, nether sprouts/wart, wither rose, lily pad, frogspawn,
mangrove propagule x2, chorus plant) still resolve to empty sets
— each belongs to its own block's item (item 59's referral,
partly discharged here).

Verification: literal comparison — all zero except KelpBlock's
`8`. `cargo check -p horizon-world --lib --tests` clean;
`level::growth_tests::` 21 passed; `cargo check -p
horizon-client --lib` clean. Rank 2 now 274/492 (90 changed,
184 no-change). work-items.json rank2.closed = 274 on
2026-11-21.

## Item 96 — game/recipe_book.rs (rank 2, NO-CHANGE + citations)

7 classes, 25 methods, 4 named: GhostSlots (+ GhostSlot),
OverlayRecipeComponent$OverlayRecipeButton (+ Pos),
RecipeBookComponent, RecipeBookTabButton,
SlotDisplay$SmithingTrimDemoSlotDisplay.

Java delta, jar-verified: the recipe-book screens differ only
by the `extract*` -> `render*` rename (`extractRenderState` ->
`render`, `extractTooltip` -> `renderTooltip`,
`extractGhostRecipe` -> `renderGhostRecipe`, `extractContents`
-> `renderContents`, `extractIcon` -> `renderIcon`,
`GuiGraphicsExtractor` -> `GuiGraphics`; `fakeItem` ->
`renderFakeItem`, `item` -> `renderItem`, `itemDecorations` ->
`renderItemDecorations`) plus visibility + requireNonNull
noise. `RecipeBookComponent.preeditUpdated` (IME pre-edit) is
26.1-only. The trim demo `resolve` is the same algorithm
inlined: 26.1's `applyDemoTransformation` helper and 1.21.11's
inlined form both do ForStacks-only, empty-if-either-side-empty,
256 random-pair draws, filter non-empty, limit 16 (1.21.11 adds
the registries-provider requirement for `applyTrim` and seeds
via `RandomSource.create` instead of `createThreadLocalInstance`).

Verified matching: `stations.rs::trim_demo` (256 draws, first
non-empty) matches `resolveForFirstStack` in both versions; the
toast is its only caller. `preeditUpdated` is already 1.21.11
(`game/ime.rs` notes `PreeditEvent` is 26.x).

Citation fixes: `extractRenderState`/`extractTooltip` ->
`render`/`renderTooltip` in recipe_book.rs (4 spots) and
recipe_book_screen.rs (module doc + 4 spots, including
RecipeBookPage — verified by this item's own diff);
stations.rs `applyDemoTransformation` -> 1.21.11's inlined
`resolve`.

Referrals: the outer SlotDisplay + its other inner types are
items 30 (DyedSlotDemo, CLOSED), 147, 350. Found here: 1.21.11
has only 10 SlotDisplay inner classes vs 26.1's 13 —
`DyedSlotDemo`, `OnlyWithComponent`, `WithAnyPotion` are
26.1-only, but the Rust's enum still carries all three and the
wire codec reads them by varint id; whether
`adapter::ids::slot_display` is correct for 1.21.11's registry
order is the codec-rewire carry-forward (item 350's audit).
Version-neutral: the demo trim's identity-hash seed is not
mirrored (the Rust draws from a fresh entity random).

Verification: literal comparison — the four screen classes
zero; SlotDisplay (outer) only the 26.1-only codec field names.
`cargo test -p horizon-client --lib --no-default-features
game::recipe_book::` runs 0 tests (no test module in
recipe_book.rs); the run compiled the lib test target cleanly.
Rank 2 now 275/492 (90 changed, 185 no-change).
work-items.json rank2.closed = 275 on 2026-11-21.

## Item 97 — level/fluids.rs (rank 2, CHANGED)

4 classes, 15 methods, 5 named: BarrierBlock, DriedGhastBlock,
LiquidBlock, FlowingFluid$SpreadContext.

Java delta, jar-verified: `LiquidBlock.getCollisionShape` in
26.1 gates on a colliding LivingEntity and asks it for its own
liquid shape (`LivingEntity.getLiquidCollisionShape`); 1.21.11
returns the fixed `SHAPE_STABLE` (bottom half,
`Block.column(16.0, 0.0, 8.0)`) for any context above it that
can stand on the fluid. The whole LiquidBlock bubble-column
apparatus is 26.1-only — `shouldBubbleColumnOccupy`,
`tryScheduleBubbleBlockColumn` (20-tick delay), the `tick`, and
the calls from onPlace/updateShape(DOWN)/neighborChanged;
1.21.11 leaves the column to Magma/SoulSand (item 68). Barrier/
DriedGhast `is(WATER)` -> `getType() == WATER` is the same
predicate; SpreadContext is visibility noise.

Fixes: `liquid_collision.rs::shape` uses the fixed bottom-half
shape with no living-entity gate (the stand check still
restricts standing to striders on lava, so no observable result
changes — the strider shape already was the bottom half). The
`entity_liquid_shape` trait method stays, documented as 26.x,
for the collision-context fixture until T4 reworks its oracle.
The LiquidBlock bubble side is removed: `liquid_block_changed`
(onPlace/neighborChanged), the Liquid updateShape arm, the
`Class::Liquid` tick arm, and both helper functions
(`should_bubble_column_occupy`, `try_schedule_bubble_column`).
`bubble_column_can_occupy` stays (it is updateColumn's own
check). The Magma/SoulSand arms are untouched.

T4 note: the collision-context fixture + Java oracle still
speak the 26.x per-entity shape protocol (`liquid_shape`
queries); the Rust no longer issues them. The oracle needs a
1.21.11 rework; the harness is not in the recorded CI profile.

Verification: literal comparison — LiquidBlock only `20` (26.1)
vs `0.0`/`16.0`/`8.0` (1.21.11); the rest zero. `cargo check -p
horizon-world --lib --tests`, `-p horizon-client-render --lib`,
`-p horizon-world --example world_probe` all clean. Bubble
tests 2 passed (the SoulSand-driven column still raises).
rustfmt --check clean on liquid_collision.rs. Rank 2 now
276/492 (91 changed, 185 no-change). work-items.json
rank2.closed = 276 on 2026-11-21.

## Item 98 — component.rs (rank 2, CHANGED)

5 classes, 23 methods, 4 named: ClickEvent$Action,
ComponentUtils, ObjectContents, AtlasSprite, PlayerSprite.

Java delta, jar-verified: the `fallback` field on object
components is 26.1-only. 26.1's ObjectContents is a 2-field
record (contents + optional fallback component) whose codec
reads `"fallback"` and whose visit prefers it; 1.21.11's is a
1-field record with no resolve at all — visit always accepts
`contents.description()` (the ObjectInfo method is renamed from
`defaultFallback()`). ComponentUtils.resolve(ResolutionContext)
is updateForEntity(CommandSourceStack, Entity) in 1.21.11 with
a hardcoded depth limit of 100 (vs the context's limit +
LimitBehavior); the walk is otherwise the same, and the limit
is unobservable through to_text_with (structural recursion,
bounded by the JSON parser). ClickEvent$Action is lambda-order
noise.

Fix: `inline_object` no longer reads a `"fallback"` field from
the object JSON — the plain-text fallback is always the computed
description (`[<name> head]` / `[unknown player head]`,
`[<sprite>]` / `[<sprite>@<atlas>]`); the resolve/contents
parameters are gone. The test that pinned the 26.1 behaviour
now expects `<[elyapluggin head] >` for a JSON fallback of
`[head]`. The translate fallback is untouched (1.21.11 has it).

Verification: literal comparison — ObjectContents only
`"fallback"` (26.1), ComponentUtils only `0` vs `100`; the
rest zero. `cargo test -p horizon-client-core --lib
component::` 12 passed. rustfmt --check clean. Rank 2 now
277/492 (92 changed, 185 no-change). work-items.json
rank2.closed = 277 on 2026-11-21.

## Item 99 — mobs/path.rs (rank 2, CHANGED)

6 classes, 13 methods, 5 named: FlyNodeEvaluator, Path,
Path$DebugData, PathType, SwimNodeEvaluator, WalkNodeEvaluator.

Java delta, jar-verified: PathType is 26 values in 1.21.11 vs
27 in 26.1 — the FIRE/DAMAGING/ON_TOP_OF_* names are
DANGER_*/DAMAGE_* in 1.21.11 (malus unchanged) and
BIG_MOBS_CLOSE_TO_DANGER (4.0) is 26.1-only. Its two uses are
26.1-only too: WalkNodeEvaluator.getNodeType returns it for
wide mobs near danger, and isDiagonalValid refuses a diagonal
step for a mob wider than one block when either neighbour's
malus is positive. SwimNodeEvaluator's BREACH test checks the
block below the candidate cell in 26.1, the candidate cell
itself in 1.21.11. Path's node-array helpers differ only in
visibility; the rest is renames.

Fixes: `diagonal_pair_valid` drops the wide-mob gate;
`swim_path_type` tests the candidate cell's own state for
WATER-pathfindability. The PathType enum and the getNodeType
OPEN override were already 1.21.11-shaped (no change).

Verification: literal comparison zero across all five classes
(the enum names are identifiers in the decompile; the javap
diff is the authority). `cargo test -p horizon-client-lib
--lib --no-default-features mobs::path::` 3 passed (the
evaluators run through the mob_ai probe on CI). rustfmt
--check clean. Rank 2 now 278/492 (93 changed, 185
no-change). work-items.json rank2.closed = 278 on 2026-11-21.

## Item 100 — bake.rs (rank 2, NO-CHANGE + citation)

5 classes, 32 methods, 3 named: FaceBakery,
ItemModelGenerator$SideDirection, ItemModelGenerator$SideFace,
UnbakedCuboidGeometry (+$1). The report mixes keys: the model
classes live under `client/resources/model/cuboid/` in 26.1 and
`client/renderer/block/model/` in 1.21.11, so the comparison was
done manually cross-package.

Java delta, jar-verified: the baking math is identical in both
versions — defaultFaceUV, bakeVertex, calculateFacing,
findClosestDirection, recalculateWinding. The difference is
material plumbing: 26.1's bakeQuad takes a Material$Baked and
computes a Transparency via the 26.1-only
computeMaterialTransparency (forceTranslucent or the sprite
contents' transparency over the UV rect); 1.21.11's takes a
TextureAtlasSprite directly. 26.1's UnbakedCuboidGeometry is
1.21.11's SimpleUnbakedGeometry (item 750, rank 4) — their bake
methods are structurally identical, including the flat-element
face skip. The SideDirection/SideFace inner classes differ only
in visibility.

No Rust fix: bake_face already mirrors 1.21.11's bakeQuad
(facing, the rotation-gated winding reorder, default_uv).
computeMaterialTransparency has no Rust mirror. force_translucent
stays per item 52's decision (T4 oracle entanglement; vanilla
1.21.11 JSON never contains the field). Citation fix:
`UnbakedCuboidGeometry.bake` -> `SimpleUnbakedGeometry.bake`.

Verification: literal comparison zero for FaceBakery and for
the geometry pair. `cargo test -p horizon-assets --lib bake::`
13 passed. Rank 2 now 279/492 (93 changed, 186 no-change).
work-items.json rank2.closed = 279 on 2026-11-21.

## Item 101 — game/structure_blocks.rs (rank 2, NO-CHANGE)

3 classes, 32 methods, 3 named: StructureBlockEntity,
StructureTemplate, StructureTemplateManager.

Java delta, jar-verified: StructureTemplate is bytecode-identical
(zero javap hunks — its +536/-579 decompile diff is Vineflower
parameter-name noise). StructureBlockEntity differs only by
`is(Object)` -> `is(Block)` in three spots. The
StructureTemplateManager diff is an internal refactor — 26.1's
`loader/` package vs 1.21.11's inlined private helpers — with no
observable behaviour change: same lookup order (generated ->
test structures, IDE-only -> datapacks), same get/save logic,
same `generated/<ns>/structures/<path>.nbt` path with the same
validation, same DataVersion-500 default (relocated from the
loader classes into the manager). Literal comparison confirms
it (only log strings and relocated path constants differ).

No Rust fix: template_entry/write_template/read_template_tag
already match 1.21.11's manager. Version-neutral gap: the Rust
never datafixes templates on load (vanilla runs the DataFixer
in both versions) — the datafix system's scope, not this item's.

Verification: `cargo test -p horizon-client --lib
--no-default-features game::structure_blocks::` 4 passed. Rank
2 now 280/492 (93 changed, 187 no-change). work-items.json
rank2.closed = 280 on 2026-11-21.

## CI proof — run 44789092726 (recorded 2026-11-21)

Branch behaviour/1.21.11 @ 0f75c53 (item 87), SUCCESS in ~2h.
Profile 5/18/0 of 23 — passing: block_properties,
component_patch, food, lan, resource_pack (same five as the
previous record). Delta verdict FAILED: D1 fail (24 dead
production tags — down from the 31 counted at publish, the
bamboo/chorus/kelp tag fixes in item 95 account for some),
D2 pass (cargo check 0 errors/0 warnings, protocol tests
440/440 on CI), D3 fail (the pin's work-items.json lagged at
195/492 at dispatch — updated to 280/492 with this record),
D4/D5 blocked (the profile-stamp quirk: profile stamped for
0f75c53, the judge's staged tree is e389abf6 — patch 0006 dead
code, Freebuff's call), D6 blocked (no playtest.md — deferred
by the owner). Recorded: pins/1.21.11/profile.json (the
published 1.16 MB profile from the delta-bar artifact, not the
1.7 KB top-level summary), delta-verdict.json, work-items.json
(280/492). VERSIONS.md row updated (harness 5/23, behaviour
290/815).

## Item 102 — containers.rs (rank 2, CHANGED)

2 classes, 11 methods, 5 named: AbstractFurnaceBlockEntity,
BaseContainerBlockEntity.

Java delta, jar-verified: the furnace serverTick genuinely differs.
(1) 26.1 nests the recipe lookup inside `if (hasIngredient)` and does
nothing when it returns null, so the cooking timer keeps its stale
value; 1.21.11 resets `cookingTimer = 0` in the else branch whenever
the furnace is in the outer `isLit() || hasFuel && hasIngredient`
arm. (2) 26.1's canBurn requires room for the full result count and
burn grows the output by the full count (`grow(result.getCount())`);
1.21.11's canBurn only requires room for 1 and burn grows by exactly
1 (`grow(1)`, javap-verified) — identical for count-1 recipes (all
vanilla cooking recipes), different only for datapack recipes
yielding 2+. NOT differences: cookingTotalTime after a burn (the
1.21.11 `getTotalCookTime` call happens before `burn()` shrinks the
input, so it sees the same recipe); the cooldown branch (`!isLit()`
is redundant in the else arm); the wet-sponge check (`Items.WET_SPONGE`
vs `Blocks.WET_SPONGE.asItem()`, same item); fuel consumption
(26.1's `consumeFuel` helper vs 1.21.11's inlined shrink + remainder,
same effect); `setRecipeUsed` gating (burn returns true whenever
reached from the tick); `isLit()` vs inlined `litTimeRemaining > 0`.
BaseContainerBlockEntity: `sendOverlayMessage` vs
`displayClientMessage(..., true)` is API shape — both show the
action bar. Literal comparison zero in both classes (the differences
are structural, so the javap diff is the authority).

Fix in `furnace.rs::tick` (the furnace tick lives there, not in
containers.rs): added `else { self.cooking_timer = 0; }` on the
recipe lookup; `can_burn` is now `out.count < max`; the burn grows a
non-empty output by 1. Two `consumeFuel` citations reworded to the
1.21.11 inline form. Two new tests
(`an_ingredient_without_a_recipe_resets_the_cooking_timer`,
`a_multi_count_recipe_only_adds_one_to_a_nonempty_output`; the
`data()` fixture gained a `data_count` variant). Version-neutral Rust
extensions left alone: the fuel-speed rescale on lighting (no vanilla
counterpart in either version) and dropping the fuel remainder when
the stack isn't used up (documented deliberate — vanilla loses it in
both versions). Tests: `cargo test -p horizon-client-lib --lib
--no-default-features furnace::` 9 passed (the furnace tests live in
the horizon-client-lib binary, not horizon-client's). rustfmt
--check clean on furnace.rs.

## Item 103 — mobs/aquatic.rs (rank 2, NO-CHANGE)

5 classes, 21 methods, 4 named: Tadpole, TadpoleModel,
AgeableWaterCreature, AbstractSchoolingFish, Squid$SquidFleeGoal.

Java delta, jar-verified: the tadpole age-lock feature is entirely
26.1-only — the `AGE_LOCKED` synced boolean, `ageLockParticleTimer`,
`setAgeLocked`/`isAgeLocked`/`setAgeLockedData`, the `AgeLocked`
save/bucket tags, the golden-dandelion interaction, and the
`!isAgeLocked()` gates on feeding and on the aiStep age increment
(literal comparison: only `"AgeLocked"` and `40` on the 26.1 side).
The Rust has no age-lock mirror anywhere (tree-wide removal done in
earlier items): aiStep ages unconditionally and converts at 24000,
feeding is ungated with vanilla's speed-up math, bucket data writes
Age only. The tadpole brain migration (`BRAIN_PROVIDER` /
`getActivities` -> `SENSOR_TYPES`/`MEMORY_TYPES` + `brainProvider()` /
`makeBrain`, same four sensors) needs no fix — the Rust cites only
`TadpoleAi`'s brain, and the activities table is item 38's. TadpoleModel
uses `entityCutout` in 26.1 vs `entityCutoutNoCull` in 1.21.11 — the
javap diff misses it (the normalizer hides the invokedynamic bootstrap
method reference, the same failure mode as the hidden string constants
of items 29/94) and the literal comparison is zero, so the decompile
is the authority; no Rust mirror (no per-model render-type table),
and the tail animation matches both versions exactly. Squid's
26.1-only `BABY_DIMENSIONS` (0.5 x 0.63, eye 0.37) was already fixed
in item 15. AgeableWaterCreature (`is(Object)` -> `is(Block)`),
AbstractSchoolingFish.tick (`getRandom()` -> `level.random`) and
SquidFleeGoal (ctor visibility) are API-shape-only; all three are
mirrored (spawn_rules `surface_water`, aquatic `school_tick`,
aquatic `SquidFlee` — including the discarded normalize).
Version-neutral gap, recorded not fixed: SquidFleeGoal.tick spawns a
BUBBLE particle at `fleeTicks % 10 == 5` in both versions, and the
Rust's goal tick has no events channel to emit one. Tests:
`cargo test -p horizon-client-lib --lib --no-default-features
mobs::aquatic::` 9 passed, 1 ignored (fixture).

## Item 104 — stations.rs (rank 2, CHANGED)

5 classes, 31 methods, 3 named: SmithingTrimRecipe, SmithingTransformRecipe,
EnchantmentMenu, GrindstoneMenu, PatchedDataComponentMap.

Java delta, jar-verified: applyTrim's material resolution differs by the
EitherHolder unwrap — 26.1 reads the `PROVIDES_TRIM_MATERIAL` component as
a `Holder<TrimMaterial>` directly, while 1.21.11 routes it through
`TrimMaterials.getFromIngredient`, which unwraps the component's
`EitherHolder` against the trim-material registry (a named holder fails
when the key is not registered; verified from the 1.21.11 jar:
`ProvidesTrimMaterial` wraps `EitherHolder<TrimMaterial>`,
`EitherHolder.unwrap` maps a key through `provider.get(key)`).
Fix: `trim_material` now takes the loaded trim-material registry and
returns None for a named material it does not hold; direct holders pass
through. The test populates the registry with copper and gains an
unknown-named-material case (no trim); the inline-pattern test's temp
data root gained a gold trim-material JSON (its fixture had no
`trim_material/` dir, so the material unwrapped to nothing).
The smithing transform is behaviourally identical — 26.1's
`TransmuteRecipe.createWithOriginalComponents` (ItemStackTemplate) vs
1.21.11's `TransmuteResult.apply` (verified: record `(item, count 1..99,
components)`, codec with the bare-string alternative; the Rust's
SmithingResult already matches) — only the citations were 26.x:
`ItemStackTemplate` does not exist in 1.21.11 (0 jar entries vs 1 in
26.1), reworded to `TransmuteResult`. NOT differences:
EnchantmentMenu.getEnchantmentList (identical), GrindstoneMenu
mergeItems/computeResult (identical), PatchedDataComponentMap.get
(inlined helper, same semantics), the SimpleSmithingRecipe/CommonInfo
rework and the cached placementInfo (API shape). Referrals: item 597
(rank 3, ProvidesTrimMaterial) owns the inline trim-material codec
shape — the Rust's `material_from_nbt` reads `palette_id`, a 26.3
field name; 1.21.11 (and 26.1) inline materials use `asset_name` +
`override_armor_assets` + `description`; item 752 (rank 4) owns the
trim-recipe serializer, incl. the unvalidated named pattern;
crafting.rs `Template` still cites `ItemStackTemplate` (item 30's
file). Tests: `cargo test -p horizon-client-lib --lib
--no-default-features stations::` 26 passed. rustfmt --check clean.

## Item 105 — level/sculk.rs (rank 2, CHANGED)

7 classes, 11 methods, 5 named: SculkVeinBlock, SculkVeinSpreaderConfig,
ChargeCursor, SculkBehaviour$1, SculkBlock, CalibratedSculkSensor
VibrationUser, SculkSpreader.

Java delta, jar-verified: pure API shape — `is(Object)` ->
`is(Block)`/`is(Fluid)`, `private` -> package-private, the 26.1
`Objects.requireNonNull` on the outer-this. Every behaviour is
identical in both versions (attemptUseCharge, getRandomGrowthState,
canPlaceGrowth, regrow, onDischarged, attemptPlaceSculk,
hasSubstrateAccess, cursor update/merge/movement, updateCursors, the
calibrated sensor's radius 16 and frequency filter; the
level-spreader constants 10/4/10/5 match the Rust's). Literal
comparison zero in all five classes.
The one fix is a dead tag in the Rust's `sculk_can_place_growth`:
vanilla counts sculk sensors and shriekers directly in both versions,
but the Rust counted members of a `minecraft:sculk_growth_inhibitors`
tag that exists in NEITHER jar (verified by listing both client jars'
data packs), so growth was never inhibited. It now counts the two
blocks directly (3+ inhibits). New test
`sculk_growth_stops_at_three_nearby_sensors` (0 sensors -> growth,
3 sensors -> none; early-returns locally without fixtures, runs on
CI). Referral: `worldgen/sculk.rs:108,558` uses the same nonexistent
tag in its own `can_place_growth` — same fix pattern, that file's
item's scope. Tests: `cargo test -p horizon-world --lib
--no-default-features sculk` 4 passed. rustfmt --check clean on
sculk.rs.

## Item 106 — level_loading.rs (rank 2, NO-CHANGE + citations)

7 classes, 19 methods, 4 named: LevelLoadingScreen, LoadingOverlay,
ConnectScreen, LevelLoadTracker (+3 state records).

Java delta, jar-verified: pure API shape — `extractRenderState` ->
`render`, `extractBackground` -> `renderBackground`,
`extractChunksForRendering` -> `renderChunks`, `extractProgressBar` ->
`drawProgressBar`, `GuiGraphicsExtractor` -> `GuiGraphics`
(`centeredText` -> `drawCenteredString`), `private` ->
package-private, the nether-portal sprite plumbing
(`getBlockStateModelSet().getParticleMaterial()` ->
`getBlockModelShaper().getParticleIcon()`), and the overlay's
no-fade clear (26.1 `clearColorOverride` vs 1.21.11 direct
`clearColorTexture` — same outcome). Literal comparison zero in all
four classes. All Rust mirrors already 1.21.11-shaped (COLORS table,
chunk-map geometry, logo layout, progress-bar fills, tracker state
machine with the 30s timeout and 500ms close delay, the ConnectScreen
cancel button). Citation fixes only (7 spots). NOT touched: the
FileFixerProgressScreen citations in this file (item 346's scope);
`LevelLoadProgressTracker` is in NO report item (report gap);
`determineLevelLoadingReason` is items 15/241's scope. Tests: `cargo
test -p horizon-client-render --lib --no-default-features
level_loading::` 12 passed. rustfmt --check clean.

## Item 107 — biome.rs + mesher.rs tint audit (rank 2, CHANGED)

20 classes, 37 methods, 2 named (`Identifier.compareTo`,
`Identifier.isValidNamespace`): ConstantInt, Identifier,
EnvironmentAttributeProbe (+ValueProbe), BlockTintSources + $1-$11,
Biome$1, Biome$TemperatureModifier (+$1, $2).

Java delta, jar-verified: `isValidNamespace` — 26.1 explicitly rejects
`".."`; 1.21.11 does not. `assertValidNamespace` error wording differs
("namespace of identifier" vs "namespace of location" — a string
difference the javap normalizer hides). ConstantInt is a Record in
26.1, a plain class in 1.21.11 (wire form identical). The probe /
Biome$1 / TemperatureModifier diffs are visibility-only plus the 26.1
`Objects.requireNonNull` javac artifact and a lambda rename.
BlockTintSources is 26.1-only; 1.21.11's `BlockColors.createDefault()`
is hardcoded per-block lambdas. The Biome outer class is item 221's
scope (only visibility/parameter-name noise plus
`fluidState.is(Fluids.WATER)` -> `getType() == Fluids.WATER` in
`shouldFreeze`).

Two real Rust bugs. (1) `BiomeInfo::from_json` read spawns from
`attributes["minecraft:gameplay/natural_mob_spawns"]` — an attribute
that exists in NEITHER 26.1 nor 1.21.11 data (verified against the
1.21.11 jar's `EnvironmentAttributes` and biome JSONs), so every
biome's spawn table was empty and natural spawning was silently
disabled. `parse_entries` now reads the `SpawnerData` form
(`type`/`weight`/`minCount`/`maxCount`, `constant: false` always —
`NaturalSpawner` draws `minCount + nextInt(1 + maxCount - minCount)`
unconditionally); spawns, `spawn_costs` and
`creature_spawn_probability` (default 0.1) come from the biome file's
top level. The `count` IntProvider arms are dropped — no 1.21.11 data
uses them. `MobSpawnSettings` is identical in both versions
(decompiled both). New test
`spawners_come_from_the_biome_file_not_an_attribute` asserts the
1.21.11 form parses and the 26.3 attribute form is ignored. (2)
`Mesher::tint` sampled `TallGrassUpper` at its own position; vanilla
samples `pos.below()` in both versions — now `(y - 1).max(0)` (the
`.max(0)` guard is required: `Tints.layers` is 16 entries, and
`saturating_sub` on a signed int saturates at `i32::MIN`, not 0 —
`0i32.saturating_sub(1)` is `-1`, caught by the test). New test
`a_tall_grass_upper_half_tints_from_the_block_below` stacks two
biomes by Y and asserts the upper half tints from the layer below,
falling back to its own layer at y=0. The rest of the tint audit
passed: 1.21.11's `createDefault` table matches the Rust's mesher
tint table registration-for-registration. Referrals:
`environment_attributes.rs` carries the 26.3-only
`gameplay/natural_mob_spawns` +
`gameplay/creature_world_gen_spawn_probability` attributes (the
environment-attributes item's scope);
`spawn_selection.rs::fortress_enemies()` sets `constant: true` for
zombified_piglin (4,4) — wrong for 1.21.11 (spawning item's scope);
`sample_count`'s doc cites `SpawnerData.count()`, which does not
exist in 1.21.11; the MISC->PIG SpawnerData quirk (unobservable in
vanilla data); the `".."` namespace mirrors in `skins.rs:653` (item
227) and `horizon-mods/src/content.rs:40`; `mod_content_biomes.rs`
still uses the attribute form (mod-content scope); `UniformInt` codec
field names are item 202's scope. Lesson: a Rust read of a 26.3-only
data form can be wrong for BOTH versions — verify the data form
against the actual 1.21.11 JSONs in the jar, not just the class diff.
Tests: `biome::` 8 passed; `mesher::tint_tests::` 5 passed, 2 ignored;
`mobs::spawn_selection::` (horizon-client-lib) 2 passed; `cargo check
-p horizon-client-lib --lib --tests` clean. rustfmt --check clean on
biome.rs and tint_tests.rs; mesher.rs flags only the pre-existing
drift at line 2145.

## Item 108 — client/renderer/item Unbaked types (rank 2, NO-CHANGE + test)

10 classes, 57 methods, 0 named: CompositeModel$Unbaked,
ItemModel$BakingContext, SelectItemModel$Unbaked,
SpecialModelWrapper$Unbaked, ConditionalItemModel$Unbaked,
SpecialModelWrapper, ItemModel$Unbaked, SelectItemModel$UnbakedSwitch,
SelectItemModel, SelectItemModel$SwitchCase.

Java delta, jar-verified: the `transformation` field is 26.1-only on all
four Unbaked records (no field, no `Transformation.EXTENDED_CODEC`, no
`Transformation.compose` in 1.21.11); `bake(BakingContext, Matrix4fc)`
-> `bake(BakingContext)`; 26.1's composite bake has the empty-list ->
`EmptyModel.INSTANCE` and single-model shortcuts, 1.21.11 always wraps;
BakingContext `SpriteGetter sprites` -> `MaterialSet materials`,
`MissingItemModel missingItemModel` -> `ItemModel missingItemModel`
(the class itself exists in both jars); 1.21.11's SelectItemModel
null-checks the selector result (defensive — `get` never returns null
in practice, the map's default is the baked fallback); SelectItemModel
fallback `orElseGet` (lazy) -> `orElse` (eager), same result;
SpecialModelWrapper drops its `transformation` field and the
`setLocalTransform` call; SwitchCase visibility only;
`SpecialModelRenderer$Unbaked<?>` -> raw (erasure); TextureSlots
package move only. Codec field names identical ("models", "on_true",
"on_false", "fallback", "when", "cases", "base", "model").

The Rust is already 1.21.11-shaped: `ItemModelNode` carries no
transformation field and decode ignores unknown fields (vanilla DFU
behaviour, documented in the module doc); `node()` iterates composite
children with no shortcuts; select falls back to the missing layer
exactly as 1.21.11's baked fallback does; special layers are identity
(`layer_matrix` composes display x local-identity); the case-list
validations mirror `validateCases` exactly (verified by decompiling
`SelectItemModelProperty$Type` from the 1.21.11 jar). No
`withTransform`/`setLocalTransform` mirror anywhere. New test pins the
core finding: a `transformation` field parses ignored on every Unbaked
record, and a composite resolves identically with and without it (an
empty composite resolves to 0 layers). Referrals: RangeDispatchItemModel
is in its own item; `ModelRenderProperties` is in no report item (item
37's note stands). Tests: `cargo test -p horizon-assets --lib
item_model::` 22 passed. rustfmt --check clean on item_model/tests.rs.

## Item 109 — copper golems (rank 2, CHANGED)

5 classes, 16 methods, 4 named: CopperGolem (8, `mobInteract`),
TransportItemsBetweenContainers (4), WeatheringCopperGolemStatueBlock
(2, `useItemOn`), OfferFlowerGoal (1, `stop`), ChestBlockEntity (1,
`playSound`).

Java delta, jar-verified: almost all API shape — ChunkPos ctor vs
`containing` + x/z fields vs methods; `mob.is(tag)` vs
`mob.getType().is(tag)` (same result); PathType renames with identical
values (DANGER_FIRE 16.0, DANGER_OTHER 16.0, DAMAGE_FIRE -1.0);
brain-provider rework (`BRAIN_PROVIDER` static ->
`CopperGolemAi.brainProvider()`/`makeBrain`); `is(Object)` -> `is(Item)`;
`getRandom()` -> `random` field. `mobInteract`, `updateWeathering`
(504000-552000 ticks), `useItemOn` and `playSound` are identical in
both versions; literal comparison zero in all five classes. Verified
from the jar: OFFER_ITEM is POPPY, OFFER_TICKS 400, the gift box is
inflate(6,2,6), `accepts_iron_golem_gift` is {copper_golem} only;
`updatePose` (parent class, decompiled manually) plays
COPPER_GOLEM_BECOME_STATUE, cycles POSE standing -> sitting -> running
-> star -> standing, and fires BLOCK_CHANGE.

Already correct: the path malus (`constructor_maluses`), the
`mobInteract` branches, the OfferFlower goal + gift, and the
transport open/close chest sound (LEFT silent, RIGHT offset). Four
version-neutral gaps fixed, all in this item's own methods and all
verified against the 1.21.11 jar: (1) the statue pose cycle was
missing — `use_golem_statue` returned false for any non-axe hand; now
a non-axe, non-honeycomb hand cycles the pose with the sound and a
BLOCK_CHANGE event; (2) an axe now frees a waxed unaffected statue
too (`getAge() == UNAFFECTED`); (3) the shear branch now fires the
SHEAR game event with the player as source; (4) the recheck-close
chest sound now skips LEFT halves and offsets RIGHT halves toward the
pair. Two new `#[ignore]`d fixture tests (pose cycle + waxed/weathered
axe). Referrals: `CopperGolemAi` is its own class (not in this item);
`CopperGolemStatueBlock` + `Pose` are in no report item. Tests: `cargo
check -p horizon-client --lib --tests` clean; `mobs::copper_golem::` 1
passed, `mobs::golem::` 10 passed. rustfmt --check clean on
copper_golems.rs and golem_gift_tests.rs.

## Item 110 — crafter (rank 2, NO-CHANGE)

4 classes, 16 methods, 4 named: CrafterScreen (12,
`extractBackground`, `slotClicked`), CrafterMenu (2,
`refreshRecipeResult`), CrafterScreen$1 (1), CrafterBlock (1,
`dispenseFrom`).

Java delta, jar-verified: API shape only — `extract*` -> `render*`
renames, `ContainerInput` -> `ClickType`, and the two-argument
`assemble(input, registryAccess)` in both `refreshRecipeResult` and
`dispenseFrom`. Literal comparison zero in all three decompiled
classes. 1.21.11's `render` calls `renderTooltip` where 26.1's
`extractRenderState` does not (the 26.x extractor framework renders
tooltips elsewhere); the disabled-slot tooltip condition is identical.
Everything else identical: slot-toggle logic (PICKUP enable/disable,
SWAP enable, UI_BUTTON_CLICK 0.4 at pitch 1.0/0.75), slot layout
(grid 26,17; result 134,35), quick-move ranges (grid -> 9..45
reversed, else -> 0..9), `dispenseFrom`/`dispenseItem` (1050 on
no/empty result, 6 crafting ticks, one-by-one vs bulk into the
container in front, spawn speed 6, 17-box advancement trigger, 1049 +
2010). `CrafterSlot` is in no report item and is bytecode-identical.
The Rust already mirrors all of it: `crafter_dispense`,
`crafter_eject`, `crafter_slot_clicked`, `crafter_tooltip`,
`screen_layout::crafter` (exact sprite numbers), and the
`Kind::Crafter` quick-move arm in menu.rs. The packet-handler citation
was verified against the 1.21.11 jar (spectator + containerId checks,
then `CrafterBlockEntity.setSlotState`). Referrals:
`CrafterBlockEntity` (serverTick, redstone signal) is item 154's;
the advancement trigger checks only the local player (pre-existing,
version-neutral). Tests: `game::machines::crafter::` 3 passed; the
`crafter` parity harness also covers it. rustfmt --check clean on
crafter.rs.

## Item 111 — block state system (rank 2, CHANGED — citation only)

10 classes, 56 methods, 0 named: StateHolder (25), StateDefinition
(14), BlockBehaviour$Properties (5), BlockBehaviour (3), BlockState
(3), StateDefinition$Factory (2), BlockStateBase$Cache (1), and the
26.1-only PostProcess, StateCollection, StateCollection$Wildcard.

Java delta, jar-verified: an internal representation rework plus
renames, no observable behaviour change. 26.1 stores states as
parallel arrays built by create*States + StateCollection;
1.21.11 stores a Reference2ObjectArrayMap + neighbours map, built
inline; enumeration order (and hence state ids) is the same, and
`cycle`/`getValue`/`setValue`/`toString` behave identically
(`getValues()` changes return type, `isSingletonState` is removed).
`getLightDampening` -> `getLightBlock` (body identical: solidRender
? 15 : propagatesSkylightDown ? 0 : 1). `PostProcess` (returns a
BlockPos, default null) -> `hasPostProcess` (boolean predicate,
default false). Properties defaults otherwise identical
(redstone-conductor / suffocating / view-blocking /
valid-spawn). 26.1's `is(Block)`/`is(TagKey)` overloads come from
the 26.x `TypedInstance` interface; 1.21.11 declares them on the
base. The Rust needs no mirror: it reads prepared data, and the
exporter writes `dampening` from `state.getLightBlock()`
(VanillaGameplayFactsExporter.java:37) — the 1.21.11 value under the
Rust's own field name. The one fix: tick.rs:1113 cited
`LightEngine.getLightDampeningInto`, which exists in NEITHER jar
(26.3-only name, item 92's failure mode) — now
`getLightBlockInto`, verified bytecode-identical in both jars.
`top_face_lets_light`'s flag-based approximation of it is
pre-existing and version-neutral. Tests: `cargo check -p
horizon-world --lib` clean; `level::growth_tests::` 21 passed.
rustfmt --check clean on tick.rs.

## Item 112 — item frames (rank 2, CHANGED)

3 classes, 15 methods, 4 named: ItemFrameRenderer (10,
`getBlockLightLevel`), ItemFrame (4, `interact`,
`removeFramedMap`), ItemFrameItem (1, `mayPlace`).

Java delta, jar-verified: the renderer is plumbing only (26.1
resolves the frame model in `extractRenderState` via
BlockModelResolver; 1.21.11 resolves it in `submit` via
`getItemFrameFakeState` + BlockRenderDispatcher — same model, same
transform; all numbers identical: 0.46875 offset, z 0.5/0.4375,
map rotation + 180, scale 0.0078125, glow lights 15728850/15728880,
glow block light max(5)). The one real delta is the entity
representation: 1.21.11's `onItemChanged` sets the framed item's
representation to the frame and `removeFramedMap` clears it; 26.1
has neither. `interact` drops the 26.x-only Vec3 parameter;
`mayPlace` is `isInsideBuildHeight` vs `!isOutsideBuildHeight`.
Literal comparison zero in all three classes. The Rust needs no
representation mirror: `maps.rs::tick_framed` already passes the
frame explicitly (documented there) and the needle reads the frame
via the `owner` passed to `resolve_item`. The fix is version-neutral
and in this item's own method: `ItemFrame::bounds()` always used
the 0.75 face, but `createBoundingBox` widens it to 1.0 with a
framed map — a framed map was targetable only over its middle 3/4
and the `survives` collision check used the narrow box. Now the
face is 1.0 when the item carries `map_id`, and the manual
widening in `frame_geometry` is removed. New unit test
`entities::item_frame_bounds_tests::` 1 passed. Tests: `cargo check
-p horizon-client --lib --tests` clean. rustfmt --check clean on
entities.rs and frames.rs.

## Item 113 — creative inventory (rank 2, NO-CHANGE)

7 classes, 24 methods, 3 named: CreativeModeInventoryScreen (15,
`extractRenderState`, `extractTabButton`, `slotClicked`),
CreativeModeTab (3), SessionSearchTrees (2), ItemStackLinkedSet$1
(2), SessionSearchTrees$Key (1), CreativeModeTabs (1),
CreativeModeTab$ItemDisplayParameters (0).

Java delta, jar-verified: renames (`extract*` -> `render*`,
`ContainerInput` -> `ClickType`, `typeHolder()` -> `getItemHolder()`,
`tags()` -> `getTags()`), `renderTooltip` in 1.21.11's `render`,
`preeditUpdated` 26.1-only, `buildContents` gains an unregistered-tab
guard, and two cursor-only differences (`insideScrollbar` drops the
`canScroll()` check in 1.21.11; 26.1 shows NOT_ALLOWED over a
too-short scrollbar, 1.21.11 shows nothing). The tab contents differ
by exactly ONE item across all 32 generators (1701 vs 1700
accepted): `GOLDEN_DANDELION` in 26.1's food-and-drink tab, absent
in 1.21.11 — verified by diffing the ordered accept sequences from
both jars. Tab contents are data: `creative-inventory.json`,
exported against the 1.21.11 jar — verified on disk (26.1's export
has golden_dandelion twice, 1.21.11's has none). The Rust's
`inside_scrollbar` (bounds + `!inventory`) matches 1.21.11's
observable behaviour: its only caller is the click path, where
vanilla also excludes the inventory tab. No NOT_ALLOWED or preedit
mirror exists to remove. Version-neutral gap: vanilla requests a
scrollbar cursor (POINTING_HAND / RESIZE_NS); the Rust requests
none there, and `widgets::Cursor` has no ResizeNs variant. Tests:
`creative::` 24 passed. rustfmt --check clean on creative.rs.

## Item 114 — axolotl (rank 2, CHANGED)

4 classes, 24 methods, 3 named: Axolotl (17, `hurtServer`,
`onStopAttacking`, `usePlayerItem`), AxolotlAttackablesSensor (5),
Axolotl$AxolotlAnimationState (1, removed in 1.21.11),
Axolotl$AxolotlLookControl (1).

Java delta, jar-verified: the brain migrates (`BRAIN_PROVIDER` +
`makeBrain(Brain$Packed)` via `AxolotlAi.getActivities()` →
`SENSOR_TYPES` + `MEMORY_TYPES` + `makeBrain(Dynamic)` via
`AxolotlAi.makeBrain`; sensors identical, 20 memory types); the
26.1-only baby animation system (7 `AnimationState` fields +
`tickBabyAnimations`/`soloAnimation`) is gone — 1.21.11 has one
`tickAnimations()` driving the four `BinaryAnimator`s (10 ticks,
IN_OUT_SINE) for every age; `BABY_DIMENSIONS` (0.5 × 0.25, eye 0.2)
is 26.1-only — 1.21.11 babies are the type dims scaled by
`getAgeScale` (0.5); `hurtServer` draws from `level().random` in
1.21.11 vs `this.random` in 26.1 (same gate, PLAY_DEAD_TICKS 200);
`AgeLocked` in bucket tags is 26.1-only; the sensor logic, tags,
`onStopAttacking`, `finalizeSpawn`, `handleAirSupply`,
`getBreedOffspring`, `usePlayerItem`, spawn rules and move control
are identical. Adult type dims jar-verified: `sized(0.75, 0.42)`,
`eyeHeight(0.2751)`.

Fixes: (1) `Animation::tick` no longer selects a baby keyframe state
— the four animators run for every age; the `states` field stays
(all stopped) for item 779 to remove with the model poses. (2) Baby
dims `Baby::Abs(0.375, 0.21, 0.09375)` → `Baby::Scale(0.5, None)`
(squid precedent, item 103): w/h were already right, but the eye
assumed an adult eye of 0.1875 where the jar says 0.2751, so the baby
eye is 0.13755. (3) `sensors()` citation `BRAIN_PROVIDER` →
`SENSOR_TYPES`. Recorded, not fixed: the `hurtServer` level-random
draw — threading the level random through the damage pipeline
(`damage`/`damage_with`/`hurt_server` + ~15 call sites) is a
cross-cutting refactor; the draw order and gates are preserved and
test-pinned. Already mirrored and verified: sensor + tags,
`on_stop_attacking` + support effects, `finalizeSpawn`,
air supply (MAX_AIR 6000, dry-out 2.0), `getBreedOffspring` (1/1200
rare, coin flip, mother's random), `usePlayerItem`,
`axolotls_spawnable_on`, move control, `AgeLocked` already dropped in
entity_nbt.rs. Tests: `mobs::axolotl::` 3 passed,
`mobs::animal_brain::` 6 passed. rustfmt --check clean on axolotl.rs,
species.rs, animal_brain.rs.

## Item 115 — panda (rank 2, CHANGED)

6 classes, 14 methods, 4 named: PandaRenderer (5,
`getTextureLocation`), PandaModel (3, `setupAnim`), Panda (3,
`mobInteract`, `tryToSit`), Panda$Gene (1), Panda$PandaBreedGoal (1),
Panda$PandaSitGoal (1).

Java delta, jar-verified: 1.21.11 has no `BabyPandaModel` and no
`BABY_TEXTURES` — babies use `PandaModel` on the
`BABY_TRANSFORMER`-baked layer (`BabyModelTransform(true, 23.0, 4.8,
2.7, 3.0, 49.0, Set.of("head"))`) and wear the adult coat; the gene
textures rename `panda_lazy.png` → `lazy_panda.png` (and worried /
playful / brown / weak / aggressive). `setupAnim` poses are
identical (sitting inlined, same numbers). `addEatingParticles`
drops the empty-hand guard and swaps the draw order (height offset
before x in 1.21.11). `mobInteract` uses `isBaby()` instead of
`canAgeUp()` (equivalent without the 26.1 age-lock) and drops the
26.1-only GOLDEN_DANDELION branch. `BABY_DIMENSIONS` (type dims ×
0.5), genes, goals, roll, sneeze, breed and sounds identical.

Fixes: (1) `body_anim.rs` baby sit branch removed — babies now sit
like adults (body 1.7407963, head PI/2); the orphaned `lerp` helper
went with it. (2) `crumbs()` draw order fixed to the 1.21.11 form
(height offset before x). New tests pin both (`a_baby_panda_sits_
like_an_adult`, `crumbs_follow_the_1_21_11_draw_order`). Already
1.21.11-shaped: `texture()` (`{gene}_panda`, no baby textures),
`interact_panda` (`isBaby()` gate, no golden dandelion anywhere),
gene tables, goals, rotations, pose. Left as-is: the empty-hand
crumb guards (unreachable — vanilla stops eating when the hand
empties) and the `BABY_TRANSFORMER` geometry (prepared layer data).
Tests: `mobs::panda::` 2 passed, `mobs::body_anim::` 7 passed.
rustfmt --check clean on panda.rs, body_anim.rs.

## Item 116 — shulker (rank 2, NO-CHANGE)

7 classes, 14 methods, 4 named: Shulker (5, `canStayAt`,
`hitByShulkerBullet`, `hurtServer`, `setRawPeekAmount`),
ShulkerRenderer (4), ShulkerModel (1), ShulkerAttackGoal (1),
ShulkerLookControl (1), ShulkerNearestAttackGoal (1),
ShulkerPeekGoal (1).

Java delta, jar-verified (javap diff 277 lines + decompile diff 935
lines, both read in full): the ONLY real change is ShulkerModel's
render type — 26.1 `entityCutoutZOffset` → 1.21.11
`entityCutoutNoCullZOffset`. It is visible only in the decompile:
the javap diff has no ShulkerModel hunk at all, the normalizer hiding
the invokedynamic bootstrap — the second case of the item-103
TadpoleModel blind spot; the decompile is the authority for render
types. Mesh and `setupAnim` identical. ShulkerRenderer is API shape
only (`SpriteId` → `Material`). Entity logic identical everywhere:
peek step 0.05/tick toward raw*0.01; `getPhysicalPeek` formula;
`getProgressDeltaAabb`; `canStayAt` (moving-piston exception,
face-full support, noCollision of the deflated fully-opened box);
`teleportSomewhere` (5 tries, ±8, y > minY, air, world border,
attach face, SHULKER_TELEPORT, peek 0, target null); `hurtServer`
(closed + arrow → false; <50% + nextInt(4)==0 → teleport; bullet →
hitByShulkerBullet); `hitByShulkerBullet` (level-random float,
failure (count-1)/5, baby keeps variant at the old position);
`setRawPeekAmount` (armor modifier swap, SHULKER_CLOSE +
CONTAINER_CLOSE at 0, else SHULKER_OPEN + CONTAINER_OPEN);
`setPos` snap + 6-tick client slide; `getRenderPosition`
((ticks-a)/6)^2*scale; `sanitizeScale` min(3.0) in both; color 16 =
null; attack goal (20 + nextInt(10)*20/2, dist^2 < 400); peek goal
(1/40, 20*(1+nextInt(3)), peek 30); nearest-attack search areas
(4.0 on the attach axis); look control atan2(-deltaRight,
deltaForward). Visibility noise only otherwise (private →
package-private, `level().getRandom()` → `level().random` — same
stream, `is(Object)` → `is(Block)`, `entity.is(TYPE)` →
`getType() == TYPE`).

Already 1.21.11-shaped: the shulker is absent from
`culls_back_faces` (mobs.rs:487 — only bat + turtle_baby cull),
i.e. NoCull, the 1.21.11 form (under the 26.1 culling form it would
have belonged in the list). `anim`/`rotations`/`texture` match the
decompile exactly; peek ease, physical peek, progress AABB,
canStayAt, teleport (including the `shulker_does_not_teleport_to`
tag and level event 2016), hurt/bullet (the level random is already
threaded through `run_pending_teleports`, unlike the axolotl gap in
item 114), setPeek sounds + container events, the +20 closed-shell
armor (`armor::values`), setPos snap + slide, look yaw, both goals,
sanitizeScale min(3.0), dims 1x1x0.5 — all mirrored. Tests:
`mobs::shulker::` 12 passed, 1 ignored. No code changes. Referral:
the shulker-box block-entity path is item 16's scope (it shares
ShulkerModel, so the NoCull form applies there too).

## Item 117 — window (rank 2, CHANGED — citations only)

2 classes, 23 methods, 3 named: Window (22,
`changeFullscreenVideoMode`, `onFramebufferResize`,
`updateFullscreenIfChanged`), Window$WindowInitFailed (1).

Java delta, jar-verified (javap diff 1111 lines + decompile diff 773
lines, both read in full): 26.1's Window is built on the 26.x
`GpuBackend` abstraction; 1.21.11's is the classic GLFW/OpenGL
window. Ctor takes the backend in 26.1 (creates its own
ScreenManager, `createWindow` via the backend with GLFW error
capture) vs a passed-in ScreenManager + inlined GLFW hints (OpenGL
3.3 core) + direct `glfwCreateWindow` in 1.21.11;
`allowedWindowMinSize` is just `Math.max(x, 1)` — identical.
`bootCrash` uses `MessageBox.error` (26.1) vs
`TinyFileDialogs.tinyfd_messageBox` (1.21.11). `updateVsync` goes to
the GpuDevice (26.1) vs `glfwSwapInterval` (1.21.11). `close()`
shuts down the ScreenManager only in 26.1. `onFramebufferResize`
fires on the same trigger in both (framebuffer size changed, new
non-zero — `getWidth()` returns the framebuffer width, so 1.21.11's
post-assign comparison is the same test) but 26.1 sets `isResized`
and calls `resizeGui()` where 1.21.11 calls `resizeDisplay()`.
`onFocus` stores the `focused` field in 26.1 (`isFocused()`) and
forwards to `eventHandler.setWindowActive` in 1.21.11 (focus owned
by Minecraft — verified in both jars' Minecraft.class).
`updateFullscreenIfChanged` (26.1) is folded into `updateDisplay`
in 1.21.11, which flips the frame first and whose `updateFullscreen`
re-flips after the mode change. `changeFullscreenVideoMode` calls
`resizeGui()` (26.1) vs `resizeDisplay()` (1.21.11), same gate.
`setMode`/`setWindowed` clamp to >= 1 and set `isResized` only in
26.1 (the clamp is a no-op on values already >= 1).
`calculateScale`/`setGuiScale`/cursor/icon/title/callbacks
identical. `isFocused()`/`isResized()`/`resetIsResized()`/
`backend()` are 26.1-only.

Already 1.21.11-shaped: `calculate_scale`/`gui_scaled_size` match
exactly; `FullscreenState` mirrors the `fullscreen` /
`actuallyFullscreen` pair (apply once per change, mode held back in
the video screen); focus lives on the Game (`Game.focused` from
`WindowEvent::Focused`), the 1.21.11 shape; no `is_resized` mirror
anywhere in the tree (resize collapses into `renderer.resize`).

Fixes (citations only): `updateFullscreenIfChanged` →
`updateDisplay` (window.rs struct doc, `apply`, `request`;
options.rs `apply_fullscreen`); `Window.updateFullscreenState` /
`Minecraft.fullscreenStateChanged` exist in NEITHER version
(26.3-only names, the item-92 class) — reworded in window.rs
(`observe`) and options.rs (`observe_fullscreen`) to say the
adoption is the winit half of the pair; `GpuSurface.PresentMode.
getSupportedVsyncMode` exists in NEITHER jar — `vsync_present_mode`
reworded as the wgpu form of `Window.updateVsync`; the struct doc's
pair name corrected to `fullscreen` / `actuallyFullscreen`. Tests:
`window::` 14 passed. rustfmt --check clean on window.rs and
options.rs. Unassigned classes recorded: VideoMode (no
`refreshRateLabel` in 1.21.11, `refreshRate` is an int, `read` uses
a regex PATTERN), FramerateLimitTracker (already at item 42),
Monitor/ScreenManager/WindowEventHandler; FramerateLimiter/
MouseHandler are item 262 (rank 3).

## Item 118 — options UI (rank 2, CHANGED)

15 classes, 43 methods, 1 named (`ConfirmScreen.keyPressed`).

Java delta, jar-verified (javap diff 1421 lines + decompile diff 1985
lines, both read in full; literal comparison across all decompiled
classes shows only `256`, the three difficulty-lock strings,
`options.worldOptions.button`, and one `0`): the one real behavioural
delta is OptionsScreen. 26.1 implements
`HasGamemasterPermissionReaction, HasDifficultyReaction`, has a 3-arg
ctor, a `difficultyButtons` (DifficultyButtons) field, and shows a
"options.worldOptions.button" button (WorldOptionsScreen) when
`canShowWorldOptions()` else the DifficultyButtons row. 1.21.11 has no
interfaces, a 2-arg ctor, and `createOnlineButton` builds the difficulty
control when `level != null && hasSingleplayerServer()`: a CycleButton
over `Difficulty.values()` (150x20, initial `level.getDifficulty()`,
onValueChange sends `ServerboundChangeDifficultyPacket`), plus a
LockIconButton (20x20) when not hardcore — 130 + 20 in an
`EqualSpacingLayout(150, 0, HORIZONTAL)` — whose press opens a
ConfirmScreen ("difficulty.lock.title", "difficulty.lock.question"
with the difficulty display name) with `lockCallback` (on Yes: send
`ServerboundLockDifficultyPacket(true)`, lock setLocked(true), both
inactive). Hardcore: difficulty button inactive, no lock button.
Otherwise the Online button (150x20) opening OnlineOptionsScreen.
Everything else is API shape: the extract* → render* renames
(item 87's tree-wide sweep), visibility/requireNonNull/access$ noise,
`AbstractContainerWidget` dropping the 26.1-only `ScrollbarSettings`
ctor param, the `lambda$init$` param type change. ConfirmScreen.
keyPressed is cosmetic: 26.1 `isEscape()` vs 1.21.11 `key() == 256`,
and `InputWithModifiers.isEscape()` is `input() == 256` in BOTH jars
(identical bytecode). KeyBindsScreen.keyPressed uses `isEscape()` in
both. Language/KeyBinds list geometry identical. LockIconButton
(decompiled from the 1.21.11 jar; not in this item's list) is 20x20,
blits its icon sprite over the whole button
(`widget/{,un}locked_button{,_highlighted,_disabled}`), narration
"narrator.button.difficulty_lock" + locked/unlocked suffix.
`ServerGamePacketListenerImpl.handleChangeDifficulty` /
`handleLockDifficulty` (1.21.11 server jar) gate on
`hasPermission(COMMANDS_GAMEMASTER)` OR `isSingleplayerOwner()` — the
SP owner is always allowed; the change calls
`setDifficulty(difficulty, false)`, so the lock is respected
server-side.

The difficulty control was entirely missing from the Rust: the Main
page always showed the Online button, even in a singleplayer world.
Fix: `OptionsUi` gains `difficulty` (default 2), `difficulty_locked`,
`hardcore`, `singleplayer` (set by `open_options_at`) and
`difficulty_confirm`; the Main page shows the difficulty CycleButton
("Difficulty: <name>") + lock icon button in a singleplayer world and
the Online button otherwise; new `WKind::IconButton` (sprite picked by
state at draw time) with focus/click/keyboard handling, new
`Act::CycleDifficulty` / `Act::LockDifficulty` and
`Event::Difficulty(u8)` / `Event::LockDifficulty`; the lock confirm
follows the PackScreen confirm pattern (ConfirmScreen geometry, Yes/No,
Escape answers no, Yes locks immediately in the UI). The game sends
`Serverbound::ChangeDifficulty` / `Serverbound::LockDifficulty` — the
packet path, not `set_difficulty_forced`: singleplayer runs on a
server thread and the window joins it through memory as a `remote`
client, so `send()` reaches the serving Game (`HostEvent::Packet` →
`lan_guest_packet`); `/difficulty` executes on the serving Game, so it
is not a model for window-side code. The guest-handler guards gained
vanilla's `isSingleplayerOwner()` bypass (`singleplayer_owner ==
Some(guest uuid)`) — without it the owner at permission 0 (no cheats)
had the packets dropped and the button was dead in the common case;
LAN-guest behaviour is unchanged. The startup menu's options handler
no-ops the two new events (no world there). Tests: 6 new render-crate
tests (layout, label/question naming via a scoped test language,
lock/hardcore states, click cycle with hard→peaceful wrap, Tab+Enter
cycle, lock-confirm Escape/No/Yes), all passing; 1 new `#[ignore]`d
game test (`difficulty_packet_tests`, needs HORIZON_TEST_RESOURCES,
compiles clean, runs on CI) pinning the owner bypass, the guest
refusal, and the lock. `cargo check` clean (render lib, client lib +
tests); rustfmt --check clean on all four edited files.

## Item 119 — block_items (rank 2, NO-CHANGE + citation)

9 classes, 23 methods, 3 named (`ShulkerBoxBlock.playerWillDestroy`,
`DecoratedPotBlock.getCloneItemStack`, `BannerBlockEntity.getItem`).

Java delta, jar-verified (javap diff 592 lines + decompile diff 1394
lines, both read in full): AttachedStemBlock.mayPlaceOn drops the
`supportBlocks` tag field and checks `Blocks.FARMLAND` directly;
BambooSaplingBlock.canSurvive renames `SUPPORTS_BAMBOO` to
`BAMBOO_PLANTABLE_ON` and its isValidBonemealTarget drops the
build-height guard; BigDripleafStemBlock.canSurvive renames
`SUPPORTS_BIG_DRIPLEAF` to `BIG_DRIPLEAF_PLACEABLE` and its
isValidBonemealTarget reshapes from the 2-arg to the 3-arg
`BigDripleafBlock.canPlaceAt`; GrowingPlantBodyBlock.
isValidBonemealTarget drops the build-height guard. All four were
already fixed by items 27/59/95 (survive.rs farmland arm, the renamed
tags, the bonemeal-valid arms). The canPlaceAt reshape is equivalent,
verified by decompiling BigDripleafBlock from both jars: 26.1's 2-arg
form delegates to `canGrowInto` (`isInsideBuildHeight &&
canReplace(fetched state)`), 1.21.11's 3-arg form is
`!isOutsideBuildHeight && canReplace(passed state)`, and every 1.21.11
call site passes the freshly fetched state. ShulkerBoxBlock.
playerWillDestroy builds the drop via the NEW 1.21.11-only statics
`getBlockByColor` / `getColoredItemStack` instead of `new
ItemStack(state.getBlock())` — the same result, since the block IS the
coloured box (another case of the triage blind spot: 1.21.11-only
methods never appear in the changed-methods dict). BannerBlockEntity.
getItem switches to `BannerBlock.byColor(baseColor)` — the same item.
DecoratedPotBlock.getCloneItemStack calls `createDecoratedPotItem`
instead of `createDecoratedPotInstance` — a rename, verified
equivalent by decompiling DecoratedPotBlockEntity from both jars (26.1
goes through the 26.x `ItemStackTemplate`, 1.21.11 sets
`POT_DECORATIONS` directly; the same stack). The
`fluidState.is(Fluids.WATER)` → `getType() == Fluids.WATER` change and
the PistonHeadBlock `is` overloads are noise.

Item 27's open referral for this item — the Rust's BigDripleafStem
PERFORM arm placing without the canPlaceAt gate, "at the ceiling it
overwrites the leaf with a stem" — is a false alarm: every entry point
into `Sim::perform_bonemeal` for the stem goes through `grow_crop`,
which checks `bonemeal_valid` (with the `dripleaf_can_place_at`
gate, height included) first; vanilla's own stem performBonemeal is
likewise ungated in BOTH versions, the gate living in
isValidBonemealTarget. The Rust mirrors vanilla exactly (the leaf
perform arm IS gated, because vanilla's leaf performBonemeal gates).
A new level test pins the resolution:
`bone_meal_refuses_a_dripleaf_stem_whose_leaf_is_at_the_ceiling`
(meal refused at the ceiling, leaf stays a leaf; the same column lower
down grows). The only code change is the comment citation in
block_items.rs (`createDecoratedPotInstance` →
`createDecoratedPotItem`). The updateShape mirrors in shapes.rs
(attached stem → stem at AGE 7, bamboo sapling, big dripleaf stem),
the pick-block table (stems → seeds, bamboo sapling → bamboo, big
dripleaf stem → big dripleaf, plant bodies → head, piston head →
piston/sticky, pot → sherds only, banner → own item) and the creative
shulker drop (`drop_creative_shulker`, creative gate at the game.rs
call site) were all verified already 1.21.11-shaped. Referrals: the
`support_tag` arm still serves NetherFungus/NetherRoots (item 757);
item 30's DecoratedPot all-brick gap is untouched; the shulker-box
block-entity path is item 16's scope. Tests: `cargo test -p
horizon-world --lib --no-default-features bone_meal` 8 passed (new
test early-returns locally without fixtures, runs on CI);
`game::block_items::tests::` 3 passed. `cargo check` clean (world lib
+ tests, client lib); rustfmt --check clean on block_items.rs, tests.rs
only its two pre-existing drift spots.

## Item 120 — lan/data (rank 2, NO-CHANGE)

6 classes, 23 methods, 3 named (`FileToIdConverter.listMatchingResources`,
`Registries.elementsDirPath`, `Registries.tagsDirPath`).

Java delta, jar-verified (javap diff 513 lines + decompile diff 992
lines, both read in full): Registries changes generic types only
(`FloatProviderType`/`IntProviderType`/`Loot*Type` wrappers; every
registry string id unchanged), drops 7 registries that are 26.1-only
(the cat/chicken/cow/pig sound variants, `trade_set`,
`villager_trade`, `world_clock`), inlines `elementsDirPath` /
`tagsDirPath` with identical results (path, `"tags/" + path`), and
loses the 26.1-only `componentsDirPath` (no Rust mirror). ListTag is
bytecode-identical (zero javap hunks; the +245/-245 decompile diff is
pure parameter-name noise). FileToIdConverter goes from a Record to a
plain class and inlines `extensionMatches` as lambdas with the
identical predicate. TagNetworkSerialization changes visibility only.
`RegistryDataLoader` was decompiled from both jars to settle the
synchronized list: 26.1 synchronizes 28 registries, 1.21.11
synchronizes 23 — the same minus the four sound variants and
`WORLD_CLOCK`, in the same order; the Rust's `SYNCHRONIZED` already
omits the sound variants. The `world_clock` entry in `SYNCHRONIZED`
was investigated and deliberately NOT removed: the internal model is
26.1-shaped by design and the adapter owns the wire difference in
both directions (the server-side adapter drops the `world_clock`
REGISTRY_DATA packet for ≤1.21.11 clients, `client_session.rs`; the
client-side adapter synthesizes it — overworld alone, clock 0 —
`translate.rs` `world_clocks()`), so the wire is already correct; in
the 1.21.11 tree the server-side registry is empty (no
`data/minecraft/world_clock/` in the 1.21.11 jar), so removing the
entry would change nothing observable today and would break the
invariant that the internal model carries the 26.1 registry set. The
clock system's behaviour is the WorldClock item's scope (item 74's
referral). The four non-vanilla entries in `SYNCHRONIZED`
(`sulfur_cube_archetype`, `decorated_pot_pattern`,
`block_transformer`, `worldgen/block_state_provider`) are
version-neutral Horizon extensions. The `tagsDirPath` / `file_order`
/ `serializeTagsToNetwork` / `ListTag.wrapIfNeeded` mirrors all match
both versions, and no citation names a 26.1-only member. Referrals:
`datapack.rs` still accepts `trade_set/` and `villager_trade/` pack
files (item 751); `RegistryDataLoader`'s load machinery is in no
report item. Tests: `lan::data::` 7 passed, 1 ignored (needs prepared
resources; the tests live in the horizon-client-lib binary, 0 in the
horizon-client lib binary); `step_1_21_11` adapter tests 5 passed,
including the two clock tests.

## Item 121 — mobs/warden_brain.rs (rank 2, CHANGED)

3 classes, 23 methods, 3 named (`WardenAi.getActivities`,
`WardenAi.updateActivity`, `GoToTargetLocation.getNearbyPos`).

Java delta, jar-verified (javap diff 596 lines + decompile diff 340
lines, both read in full): the brain migration, API shape only — the
same pattern as items 82, 103 and 114. 26.1's `getActivities` returns
`List<ActivityData<Warden>>`; 1.21.11 has `makeBrain(Warden, Dynamic)`
via `Brain.provider(MEMORY_TYPES, SENSOR_TYPES)` with void
`init*Activity(Brain)` methods. `Warden` itself: 26.1 has a static
`BRAIN_PROVIDER` + `makeBrain(Brain$Packed)`; 1.21.11 has no
`BRAIN_PROVIDER`, only `makeBrain(Dynamic<?>)` (javap of `Warden.class`
from both jars). `updateActivity` takes the `Warden` in 1.21.11 instead
of the `Brain`, with the identical priority order (EMERGE, DIG, ROAR,
FIGHT, INVESTIGATE, SNIFF, IDLE). All activity content is identical in
both versions (core/emerge/dig/idle/roar/fight/investigate/sniff,
priorities, behaviours, conditions); `GoToTargetLocation.getNearbyPos`
changes only `level.getRandom()` → the `level.random` field;
`WardenEntitySensor.doTick` is identical (the `is(EntityType.PLAYER)` →
`getType() == EntityType.PLAYER` change is the same test). Constants
verified from the 1.21.11 decompile: EMERGE_DURATION 134,
SNIFFING_DURATION 84, DIGGING_DURATION 100, DIGGING_COOLDOWN 1200,
DISTURBANCE_EXPIRY 100 — all already match the Rust.

Changes: the `getActivities` / `BRAIN_PROVIDER` / `getRandom`
citations are reworded to the 1.21.11 form (`WardenAi.makeBrain`,
`SENSOR_TYPES`/`MEMORY_TYPES`, `Level.random`) — the item-82 referral,
now discharged. And `set_disturbance` now gates on the world border:
vanilla's `WardenAi.setDisturbanceLocation` checks
`WorldBorder.isWithinBounds(position)` first in both versions, and the
Rust checked only the anger/attack-target gates. Verified from the
1.21.11 jar that `isWithinBounds(BlockPos)` is the block's corner with
no margin — exactly `BorderGeometry::is_within_bounds(x, z, 0.0)`, the
shulker idiom. All three Rust call sites mirror vanilla
`setDisturbanceLocation` callers (javap over the 1.21.11 warden
classes): `do_push` ↔ `Warden.doPush`, `receive_vibration` ↔
`Warden$VibrationUser.onSignalReceive`, Sniffing do_stop ↔
`Sniffing.doStop` (the behaviours are top-level classes in
`behavior/warden/` in 1.21.11, not `WardenAi$` inners). The game passes
the real border into MobWorld, so the gate is live in-game. New
`#[ignore]`d test pins it (a 4-block border centred on the origin: a
disturbance at [10, 64, 10] must not land, one at [0, 64, 0] must).
The Rust has no NEAREST_VISIBLE_NEMESIS slot — no warden sensor or
behaviour writes it in either version, so it is vestigial, no action.
Referrals: monster_brain.rs:1-2,99 still cite `BRAIN_PROVIDER` (item 82
fixed the `getActivities` citations but missed the module doc and
sensor-order comment); villager_brain.rs:1,312 cite
`Villager.BRAIN_PROVIDER` (the villager-brain item's scope). Tests:
`mobs::warden_brain::` 2 passed, 2 ignored (needs prepared resources);
`mobs::warden::` 8 passed.

## Item 122 — spectator_menu.rs (rank 2, CHANGED)

8 classes, 22 methods, 3 named (`SpectatorGui.extractHotbar`,
`SpectatorGui.extractAction`, `SpectatorMenuItem.extractIcon`).

Java delta, jar-verified (javap diff 332 lines + decompile diff 569
lines, both read in full): almost entirely the `GuiGraphicsExtractor`
→ `GuiGraphics` rename (extract* → render*, `text` → `drawString`,
`textWithBackdrop` → `drawStringWithBackdrop`,
`PlayerFaceExtractor.extractRenderState` → `PlayerFaceRenderer.draw`),
plus visibility-only changes (SpectatorMenu's sprites, texts and
`page` field private → package-private) and one unobservable change
(TeamSelectionItem.create picks the icon member with
`RandomSource.create()` instead of `createThreadLocalInstance()` —
both non-deterministic; the Rust picks by SystemTime nanos). All
behaviour identical: fade, hotbar/action geometry, scroll, selectSlot,
paging, player filter + UUID ordering, team fill + face. Constants
verified: FADE_OUT_DELAY 5000L / FADE_OUT_TIME 2000L exist as fields
in both versions, HOTBAR_SPRITE "hud/hotbar", all eight spectatorMenu
lang keys in the 1.21.11 jar.

Changes: the three citations reworded to the 1.21.11 form
(`renderHotbar` / `renderAction` / `renderIcon`). And the team icon's
colour square now fades with the menu: vanilla fills (1,1)-(15,15)
with `ARGB.colorFromFloat(alpha, r * brightness, ...)` while the Rust
filled at alpha 255, so the square stayed opaque while everything
else faded out. Version-neutral, in this item's named class, small
fix (the item-121 precedent); the wrong `ARGB.scaleRGB` comment goes
with it. New test pins the fill colour at alpha 0.5 and at brightness
0.25. Checked, no action: PlayerMenuItem is in no report item
(decompiled from both jars — rename only; its face draws with
`ARGB.white(alpha)` and no brightness scaling, but Player items can
only ever be enabled, so the Rust's brightness tint equals
white(alpha) there); RootSpectatorMenuCategory is in no report item
(prompt "spectatorMenu.root.prompt", matches). Referral:
game/spectator.rs:110 still cites `Hud.extractHotbar` / `extractAction`
(the item-87 tree-wide sweep). Tests: `spectator_menu::` 1 passed
(new; the module had no tests before).

## Item 123 — level/survive.rs (rank 2, NO-CHANGE)

3 classes, 12 methods, 4 named (`VineBlock.getUpdatedState`,
`VineBlock.randomTick`, `ScaffoldingBlock.getDistance`,
`ScaffoldingBlock.isBottom`).

Java delta, jar-verified (javap diff 188 lines + decompile diff 782
lines, both read in full): zero behaviour change — the javap diff is
the `is`-overload, `getType()` and `getRandom()`-field noise, the
decompile diff is parameter-name noise, and a literal comparison over
all three classes returns zero string or numeric differences. All
mirrors already match both versions: GrowingPlant canSurvive (with
the kelp magma hardcode already 1.21.11-shaped from item 95),
Scaffolding getDistance + canSurvive (`distance < 7`), Vine
getUpdatedState + canSupportAtFace + canSurvive (`countFaces > 0`).
`isBottom` is mirrored exactly by `scaffolding_is_bottom` in tick.rs
(used by place.rs and tick.rs); `randomTick` by `vine_random_tick` in
tick.rs (item 95's file). No citation names a 26.1-only member.
Tests: `level::growth_tests::` 21 passed (smoke check; no code
changed).

## Item 124 — mobs/anim.rs (rank 2, CHANGED)

10 classes, 21 methods, 3 named (`DrownedRenderer.getArmPose`,
`AnimationUtils.animateZombieArms`, `EndermanModel.setupAnim`).

Java delta, jar-verified (javap diff 950 lines + decompile diff 1552
lines, both read in full): one real behaviour change — 26.1's
`animateZombieArms` returns early for a baby holding a main-hand
item, 1.21.11 has no baby guard. The rest is API shape and layers:
DrownedRenderer loses the baby texture and BabyDrownedModel (always
the adult texture, plain DrownedModel in the baby layer);
DonkeyModel / EquineSaddleModel / SheepModel gain the 1.21.11 baby
layers (`createBabyLayer`, `createSaddleLayer(float, boolean)`,
SheepModel.BABY_TRANSFORMER); PiglinModel becomes concrete with a
`setAllVisible` override; EndermanModel's carried-block check goes
from `BlockModelRenderState.isEmpty()` to a BlockState null check
(same arms); QuadrupedModel loses the render-type ctor;
HumanoidMobRenderer's `getAttackArm` inverts its comparison with
identical meaning.

Changes: the zombie-arms block drops the 26.1-only baby guard
(babies now raise their arms like adults), and the orphaned
`Hum.holding` field goes with it (field, default, setter). New test
pins a baby zombie's raised arms at rest. Checked, no action: the
drowned texture is already 1.21.11-shaped (one texture for both
ages, recorded in render.rs); the enderman carried-block arms match
exactly; the equine/sheep baby geometry lives in the prepared layer
data; PiglinModel.setAllVisible needs no mirror (no per-part
visibility toggling in the Rust); HumanoidModel's 26.1-only extra
parts belong to the HumanoidModel item. Version-neutral gap
recorded: `humanoid()` bobs the arms for every humanoid while
vanilla's `bobArms` runs only for zombies — not a delta, left for
its own item. Tests: `mobs::anim::` 12 passed (1 new).

## Item 125 — game/books.rs (rank 2, CHANGED)

6 classes, 19 methods, 3 named (`BookViewScreen.handleClickEvent`,
`LecternScreen.bookChanged`, `LecternScreen.pageChanged`).

Java delta, jar-verified (javap diff 352 lines + decompile diff 832
lines, both read in full): no behaviour change. The whole delta is
the GuiGraphicsExtractor → GuiGraphics rename, a constant-true
guarded pattern switch in 26.1's `handleClickEvent` (plain switch
in 1.21.11, same semantics), `bookChanged`/`pageChanged` going
private → package-private, and a requireNonNull compiler artifact
in `LecternMenu$1`'s ctor.

Changes: `save_book` now drops every trailing empty page, as
vanilla's `eraseEmptyTrailingPages` does — the Rust kept at least
one, so an all-blank book sent `[""]` where vanilla sends `[]`
(version-neutral, in this item's named class; extracted as
`drop_trailing_empty_pages` with a unit test). The page-indicator
comment no longer says "26.3's en_us template" (1.21.11's value is
the same positional form). Checked, no action: `book_follow_link`
mirrors `handleClickEvent` exactly; the lectern book/page sync,
turn-page visibility and sound, editor limits (1024 chars, 14
lines, 100 pages, 15-char title), PageButton geometry, and the
menu engine's lectern buttons all match both versions, including
the `setPage` clamp's count = 0 case. `LecternBlockEntity.resolveBook`
(the item-32 referral) exists in both versions and has no Rust
mirror — server-side book-content resolution, recorded not fixed.
Version-neutral nit recorded: Take Book resets the menu's page to
0 in the Rust while vanilla leaves it — unobservable, out of this
item's file scope. Tests: `game::books::save_book_tests::` 1 passed
(new; the module's other 3 tests are `#[ignore]`d, fixture-bound).

## Item 126 — game/hand.rs (rank 2, CHANGED)

3 classes, 19 methods, 3 named (`AttackRange.defaultFor`,
`AttackRange.effectiveMinRange`, `PiercingWeapon.attack`).

Java delta, jar-verified (javap diff 328 lines + decompile diff
343 lines, both read in full): one behaviour change —
`AttackRange.effectiveMinRange` returns 0.0 for a spectator in
1.21.11, where 26.1 gave it the creative or survival minimum
(`effectiveMaxRange` has no spectator branch in either version).
The rest is shape: the record accessors rename reach → range
while the JSON codec names stay `min_reach`/`max_reach`/etc. in
both versions; `PiercingWeapon.attack` switches its range source
to `entityAttackRange()` (the active item — use item, else main
hand) and its post-attack hook to `lungeForwardMaybe()` — both
behaviourally identical to the 26.1 forms (the STAB path always
uses MAINHAND while not using, and both hooks apply the same
`POST_PIERCING_ATTACK` effect list); `defaultFor` keeps its
values; SimpleType goes private → package-private.

Changes: `attack_reach` / `attack_range_contains` take a
spectator flag forcing the minimum to 0, threaded through
`within_attack_range`, `held_range_allows`, `attack_range_pick`
and `crosshair_living` — reachable via a spectator's
`spectate_hit_entity` click with a spear held. The bounds test
gains the spectator assertions. Checked, no action: the JSON
keys, the `entityAttackRange` citation in `within_attack_range`
and the `lungeForwardMaybe` citation in `jab()` were already the
1.21.11 form. Naming nit recorded: spear.rs's
`post_piercing_attack()` is the 26.1 hook name for 1.21.11's
`lungeForwardMaybe` — a cosmetic rename for the spear item's
scope. Tests: `game::hand::tests::` 4 passed.

## Item 127 — game/integrated.rs (rank 2, CHANGED)

2 classes, 19 methods, 3 named (`IntegratedServer.initServer`,
`IntegratedServer.publishServer`, `IntegratedServer.tickServer`).

Java delta, jar-verified (javap diff 553 lines + decompile diff
484 lines, both read in full): 1.21.11 keeps the singleplayer
owner's player tag in memory on save (`IntegratedPlayerList.save`
+ `getSingleplayerData`) and reads it from level.dat
(`getLoadedPlayerTag`) for the level-load focus position, where
26.1 loads it from playerdata/ on disk; the low-disk warning is
restructured — 26.1's base-class `warnOnLowDiskSpace` /
`sendLowDiskSpaceWarning` don't exist in 1.21.11, where
`IntegratedServer.saveEverything` calls a private
`warnOnLowDiskSpace` after every save (and the chunk-failure
reports call it too); 26.1 saves everything at the end of
`initServer` and refreshes chat abilities on `publishServer`,
1.21.11 does neither; `forceGameTimeSynchronization` →
`forceTimeSynchronization` (the WorldClock item's scope);
`onServerCrash` relays via `minecraft.delayCrashRaw`; the system
report says "Integrated Server (map_client.txt)"; several
methods go protected → public.

Changes: `save_game_with` (the `saveEverything` mirror) now warns
on low disk space after every successful integrated save — before,
the toast only appeared on a chunk failure. chunk_io.rs citations
reworded to the 1.21.11 form (`IntegratedServer.warnOnLowDiskSpace`,
no `sendLowDiskSpaceWarning`). No change needed in integrated.rs
itself (no init save, no publish chat refresh — already the
1.21.11 form). Checked, no action: `store_owner` / `load_owner`
already mirror the level.dat owner mechanism; `save::low_disk_space`
already mirrors the 64 MiB check; the window toasts from the queue
and the dedicated server logs instead. Gap recorded: no
`selectLevelLoadFocusPos` mirror — `prepare_levels` waits on saved
tickets rather than loading around the owner's saved position.
Tests: `chunk_io::` 3 passed.

## Item 128 — game/nether_portal.rs (rank 2, CHANGED)

2 classes, 19 methods, 3 named (`ChunkPos.containing`,
`ChunkPos.pack`, `BaseFireBlock.onPlace`).

Java delta is API shape only: ChunkPos goes record → plain
class (`containing`/`unpack` statics become ctors, `pack` →
`asLong`/`toLong`, the `getRegionX(long)`/`getRegionZ(long)`
statics are gone, all values identical); BaseFireBlock differs
only by `is(Object)` → `is(Block)` noise and `getRandom()` →
`random`.

The real find: `portal_frame` read the
`minecraft:nether_portal_frame` tag, which exists in NEITHER
jar (26.3-only) — a nonexistent tag resolves to the empty set,
so no portal frame was ever found and fire could never light a
portal. Vanilla's `PortalShape.FRAME` is `state.is(Blocks.OBSIDIAN)`
in both versions (verified from the 1.21.11 decompile and the
26.1 javap lambda); the predicate now checks obsidian directly.
New level test `fire_lit_inside_an_obsidian_frame_creates_a_portal`
(runs on CI; early-returns locally without fixtures). Citations
reworded: `ChunkPos.containing` → `ChunkPos(BlockPos)`,
`ChunkPos.pack(x, z)` → `ChunkPos.asLong(x, z)`; the Rust's own
`pack`/`unpack` API names stay. Checked, no action:
`base_fire_on_place` matches `onPlace` exactly;
`find_empty_portal_shape` tries the preferred axis then the
other, as vanilla's `findPortalShape` does;
`fire_can_be_placed_at` matches `canBePlacedAt`/`isPortal`;
`portal_empty` is fine (`BlockTags.FIRE` exists in 1.21.11).
Recorded: the game-level `try_light_portal` is an uncalled
duplicate of the live `base_fire_on_place` path (cleanup, not
fixed); the rest of the PortalShape mirror belongs to the
PortalShape item.

## Item 129 — merchant.rs (rank 2, CHANGED)

6 classes, 29 methods, 2 named (`VillagerProfession.getTrades`,
`SetEnchantmentsFunction.run`).

Java delta: the data-driven trade system is 26.1-only —
`tradeSetsByLevel` + `getTrades(int)` on the profession,
`TradeCost` / `TradeSet` / `VillagerTrade` classes, and 746
`trade_set`/`villager_trade` data files in the 26.1 jar vs 0 in
1.21.11, where trades are hardcoded `VillagerTrades` listings.
`SetEnchantmentsFunction.run` is identical in both versions.

Changes: loot.rs `set_enchantments` now mirrors `run` — `add`
adds to the existing level (was ignored, always set) and every
level clamps to 0..255 (was unclamped); new test pins both.
merchant.rs module doc now states the data-driven registries are
26.1-only and `Trades` loads empty here. Recorded, item 751's
scope: the Rust's offer generation is the 26.1 engine, so with
no trade data every villager and wandering trader gets zero
offers; porting the hardcoded `VillagerTrades` listings (item
751, rank 4) is what restores trading. Report gaps:
`WanderingTraderTrades` and the `VillagerTrades` outer class
(TRADES table, updateTrades mechanics) are in no report item.
Tests: `loot::` 35 passed (1 new).

## Item 130 — mobs/finalize.rs (rank 2, CHANGED)

2 classes, 9 methods, 4 named (`Zombie.finalizeSpawn`,
`Zombie.handleAttributes`, `Zombie.hurtServer`,
`Zombie.wantsToPickUp`).

Java delta: 1.21.11's `handleAttributes(float)` never heals —
26.1 heals the zombie to full after the leader bonus (unless
CONVERSION/LOAD/DIMENSION_TRAVEL), so a 1.21.11 leader zombie
spawns with its old health under its raised maximum.
`onOffspringSpawnedFromEgg` is 26.1-only (no Rust mirror to
remove). BABY_DIMENSIONS is the item-64 fix (scale 0.5, eye
0.93). TropicalFish's diff is parameter-name noise; its
`finalizeSpawn` and the group-data variant are identical.

Changes: `zombie_attributes` drops the heal and its now-unused
`reason` parameter (three call sites updated); new test pins a
leader spawning with old health under a raised maximum.
Checked, no action: ZombifiedPiglin's 0.0 reinforcements with
no draw (verified in the 1.21.11 jar); the tropical-fish
finalize (0.9 common, else lone + random pattern/colours);
the zombie finalize order and draw order. Tests:
`mobs::finalize::` 20 passed (1 new).

## Item 131 — crash.rs (rank 2, CHANGED)

4 classes, 18 methods, 3 named (`BlockableEventLoop.relayDelayCrash`,
`Util.onThreadException`, `Util.safeReplaceOrMoveFile`).

Java delta: the delayed-crash mechanism is 26.1-only
(`delayedCrash`, `propagatesCrashes`, `delayCrash`,
`relayDelayCrash`, `hasDelayedCrash` — none exist in 1.21.11).
`onThreadException` in 1.21.11 logs a plain worker-thread
exception and carries on (only a ReportedException prints its
report and exits -1); 26.1 relays a crash report to the main
thread. `safeReplaceOrMoveFile` is identical in both versions;
`safeMoveFile` is 26.1-only. ServerWatchdog differs only by the
gamerules accessor path (same value).

Changes: a worker-thread panic is now logged, not reported, and
the game carries on — the pool already contains the panic, so
only the hook's exit(101) ended the game. `worker_panic_report`
removed with the branch. Checked, no action: the
`restore_level_data_from_old` deviation stands (the function is
identical in both versions); the watchdog mirror is identical.
Tests: `crash::` 11 passed.

## Item 132 — game/native_mob_draw.rs (rank 2, CHANGED, citations only)

1 class, 8 methods, 4 named (`EntityRenderer.extractRenderState`,
`EntityRenderer.getPackedLightCoords`, `EntityRenderer.submit`,
`EntityRenderer.submitNameTag`).

Java delta: in 26.1 the below-name score moved up to the base
`EntityRenderer` (`Entity.belowNameDisplay()` + `scoreText`,
drawn within 10 blocks); in 1.21.11 it lives on the player side
only (`AvatarRenderer.submitNameTag` from
`ClientAvatarEntity.belowNameDisplay()`). Renames:
`LightCoordsUtil.pack` → `LightTexture.pack`,
`Lightmap.getBrightness` → `LightTexture.getBrightness`,
`submitNameDisplay` → `submitNameTag`.

Changes: the two citations reworded (`LightTexture.pack`,
`LightTexture.getBrightness`; both formulas verified against
the 1.21.11 bytecode). Checked, no action: the below-name score
is already players-only in names.rs with the 1.21.11 geometry;
`getPackedLightCoords` is mirrored; the other `LightCoordsUtil`
citations stay with item 191. Tests: `cargo check
-p horizon-client-render --lib` clean.

## Item 133 — game/text_displays.rs (rank 2, CHANGED, citations only)

6 classes, 28 methods, 2 named (`DisplayRenderer.getBlockLightLevel`,
`DisplayRenderer.getShadowRadius`).

Java delta: `BlockModelResolver` + `CompositeBlockModel` are
26.1-only — 1.21.11's BlockDisplayRenderer calls
`submitBlock(poseStack, blockState, light, NO_OVERLAY,
outlineColor)` and the model resolves at render time via
`BlockRenderDispatcher.renderSingleBlock` →
`ModelBlockRenderer.renderModel` (seed 42, verified from the
bytecode), with the special model from
`SpecialBlockModelRenderer.renderByBlock` (NONE) inside
`SubmitNodeCollection.submitBlock`. `LightCoordsUtil.sky/block`
→ `LightTexture.sky/block`; `mulPose(Transformation)` →
`mulPose(getMatrix())`.

Changes: the two citations in `block_item_display_geometry`
reworded to the 1.21.11 path. Checked, no action: `display_light`
mirrors the brightness override; the text background reads the
option directly, the 1.21.11 form; the item display is identical
in both versions. Tests: `game::text_displays::tests::` 5 passed.

## Item 134 — mod_content_tests.rs / WorldGenRegion + NoiseBasedChunkGenerator (rank 2, CHANGED)

2 classes, 8 methods, 4 named (`WorldGenRegion.ensureCanWrite`,
`WorldGenRegion.getCurrentDifficultyAt`, `WorldGenRegion.setBlock`,
`NoiseBasedChunkGenerator.addDebugScreenInfo`).

Java delta: 1.21.11's `WorldGenRegion.destroyBlock` drops the
resources when asked (26.1 just sets air); `setBlock`
post-processing is `BlockState.hasPostProcess` marking the set pos
(26.1's `getPostProcessPos` exists in no 1.21.11 class);
`addDebugScreenInfo` puts N (final density) last (26.1 put the
interpolated density first via the 26.1-only
`getInterpolatedNoiseValue`).

Changes: (1) `NoiseRouter::from_json` accepts 1.21.11's
`preliminary_surface_level` for the surface field when
`chunk_surface_level` is absent — verified against the real 1.21.11
overworld router JSON, decoding failed with
`Err(Field("chunk_surface_level"))` before the fix, so the tree
could not load 1.21.11's own noise settings at all. (2)
`debug_density_line` falls back to the hardcoded 1.21.11 router
line (T V C E D W PV PS N) when the settings carry no
`debug_functions`, with a new `peaks_and_valleys` (verified from
the bytecode); the mod-content test now expects the 1.21.11 label
order. (3) `getPostProcessPos` citation → `hasPostProcess` (the
mechanism was already 1.21.11-shaped: brown/red mushroom + magma,
all `Blocks.always`, nothing marks above). (4) Honest gap notes:
no loot is evaluated during worldgen, so the 1.21.11 destroyBlock
drops are lost (only the end podium destroys with drops during
generation). Tests: `worldgen::router::` 2 passed (1 new),
`peaks_and_valleys_matches_vanilla` 1 passed (new), `cargo check
-p horizon-client --lib --tests` clean.

## Item 135 — horizon-command/src/args.rs (rank 2, CHANGED, citations only)

34 classes, 48 methods, 0 named in Horizon.

Java delta: SlotRanges names the 300+8 range `villager.` in
1.21.11 (`mob.inventory.` in 26.1, with the 26.1-only
`MOB_INVENTORY_SLOT_OFFSET`/`SIZE` constants); both suggestion
providers spell the method `getCustomTabSugggestions` (sic) in
1.21.11; everything else is visibility noise. The literal
comparison across all 34 classes returned exactly one difference
(the slot-range prefix).

Changes: three citations reworded to the 1.21.11 typo'd name
(registries.rs, chat.rs, chat_tests.rs). Checked, no action:
`slot_ranges()` already uses `villager.` and matches 1.21.11's
table exactly. Tests: `chat::tests::` 27 passed.

## Item 136 — mobs/strider.rs (rank 2, CHANGED)

2 classes, 7 methods, 4 named (`Strider.floatStrider`,
`Strider.getLiquidCollisionShape`, `Strider.tick`,
`StriderGoToLavaGoal.isValidTarget`).

Java delta: the suffocation rule flipped — 1.21.11 suffocates when
`!inWarm || riddenStrider.isSuffocating()` (26.1: `!inWarm &&
!ridingAWarmStrider`); `FIRE_IN_NEIGHBOR`/`FIRE` →
`DANGER_FIRE`/`DAMAGE_FIRE`; `floatStrider` tests
`LiquidBlock.SHAPE_STABLE` (the `getLiquidCollisionShape` override
is gone; same half-block shape).

Changes: `pre_tick` now sets the flag to `!in_warm ||
on_suffocating_strider`; the float citation reworded to
`SHAPE_STABLE`. New test `a_ridden_cold_strider_chills_its_rider`
(warm rider on a cold mount is cold; cold rider on a warm mount is
cold; warm on warm is warm). Checked, no action: the path maluses
already use the 1.21.11 names; `lava_target` matches
`isValidTarget`. Tests: `mobs::strider::` 1 passed (new;
early-returns locally without fixtures).

## Item 137 — mobs/trial_spawner.rs (rank 2, CHANGED)

4 classes, 17 methods, 3 named (`WeightedList.isEmpty`,
`TrialSpawnerState.spawnOminousOminousItemSpawner`,
`TrialSpawnerState.tickAndGetNext`).

Java delta: `WeightedList.isEmpty` is `items.isEmpty()` in 1.21.11
(26.1: total weight 0); `FlameParticle.getLightColor` inlines the
emission as `(int)(progress * 15 * 16)` on the block channel;
`PortalParticle.getLightColor` puts its fourth-power emission on
the SKY channel in 1.21.11 (26.1's `addSmoothBlockEmission` put it
on block); `tickAndGetNext` is renumbering only.

Changes: `has_mob_to_spawn` and `apply_ominous` test
`!spawn_potentials.is_empty()`; `AgeEmission` uses `* 15.0 *
16.0`; Portal moved to the sky channel (`AgeEmissionSky4`, also
`* 15.0 * 16.0`, unclamped); the `AgeEmission4` variant is removed.
New test `flame_and_portal_light_go_to_their_12111_channels`.
Tests: `mobs::trial_spawner::` 5 passed, `particles::` 58 passed
+ 1 ignored, new light test passes.

## Item 138 — level/pistons.rs (rank 2, NO-CHANGE)

2 classes, 7 methods, 4 named (`checkIfExtend`, `isPushable`,
`moveBlocks`, `triggerEvent`). The javap diff is the `is(Object)`
-> `is(Block)` overload noise plus `triggerEvent` renumbering
(26.1 stores `level.getRandom()` in a local; 1.21.11 reads the
`level.random` field inline — same stream, same pitch draws);
the literal comparison returns zero differences across both
classes. The Rust already mirrors the Java exactly (flags 67/276,
pitches, volumes, game events, the sticky-pull gate). Test:
`level::tests::pistons_push_and_sticky_pistons_pull` 1 passed.

## Item 139 — world/level/storage (rank 2, NO-CHANGE)

11 classes, 47 methods, 0 named. Java delta: `LevelDataAndDimensions`
drops the `WorldDataAndGenSettings` inner record (holds `WorldData`
directly); `LevelSummary.requiresFileFixing` is 26.1-only and gone in
1.21.11; `ServerLevelData` reshuffles its accessors onto the interface
and gains a "Level weather" crash-report line; the world-delete visitor
uses `Files.delete` (throws on a vanished file) where 26.1 used
`deleteIfExists`. None of it has a Rust mirror (no LevelSummary struct,
no backup-status/primary-action mirror, no level-data crash category;
the Rust deletes worlds with `remove_dir_all`). Tests:
`save::tests::` 71 passed, 3 pre-existing datafix-upgrade failures on
the unmodified tree.

## Item 140 — gfx/animate.rs (SpriteContents) (rank 2, CHANGED, citations only)

3 classes, 16 methods, 3 named (`computeTransparency`,
`increaseMipLevel`, `transparency`). Java delta: the whole
sprite-transparency mechanism is 26.1-only — `NativeImage.
computeTransparency`, `SpriteContents.transparency` /
`computeTransparency`, `FaceBakery.computeMaterialTransparency`
and `Layer.bySprite` are all gone in 1.21.11 (zero references in
the client.jar). 1.21.11 picks block-quad layers from the block's
render type, item-model layers by atlas (items atlas always the
translucent sheet), and terrain/breaking particle layers by atlas
(TERRAIN vs ITEMS). `generateMipLevels` drops the transparency
parameter; AUTO now resolves via `hasTransparentPixel` (any alpha
== 0) — the Rust's `has_transparent` already matches exactly.
Frame textures get `byMipLevel.length + 1` mip levels in 1.21.11
(no Rust mirror — one array texture, not per-frame textures).
Changes: six doc rewords (sprites.rs, catalog.rs, hud.rs,
particles.rs, item_render.rs, pack_atlas.rs) — the per-quad /
per-sprite pixel scans are this engine's own optimizations, kept
because they are pixel-identical. Referrals: the particle-layer
switch to items 354/192 (from item 28), the item-layer decision to
the item-render item. Tests: `cargo check -p
horizon-client-render --lib` clean.

## Item 141 — game/explosion.rs (rank 2, NO-CHANGE)

2 classes, 6 methods, 4 named (`hurtEntities`,
`shouldAffectBlocklikeEntities`, `useItemOn`, `wasExploded`).
The delta is API-shape noise: the tag/type checks move from
`entity.is(...)` to `entity.getType().is(...)` / `==`, the
`is` overloads, `getRandom()` -> the `random` field, and
`sendOverlayMessage` -> `displayClientMessage(msg, true)` (the
same overlay message and PASS). The Rust already mirrors it
all (`ignite_tnt` shows the disabled overlay and returns PASS).
Recorded, not fixed: the redirectable-projectile owner transfer
is missing from the Rust's blast loop in both versions'
mirror. Tests: `game::explosion::tests::` 11 passed, 2 ignored.

## Item 142 — game/menu_clicks.rs (rank 2, NO-CHANGE)

9 classes, 16 methods, 3 named (`clicked`, `doClick`,
`getQuickCraftPlaceCount`). The delta is the `ContainerInput`
-> `ClickType` rename (the Rust already uses `ClickType`) and
the `getQuickCraftPlaceCount` signature change (int count ->
the slot set, same values); the doClick bodies are otherwise
identical. The Rust's quick-craft distribution already matches
1.21.11, including the stage-1 `>` / stage-2 `>=` count gates.
Tests: `menu::tests::` 53 passed.

## Item 143 — item_sync.rs / Raid (rank 2, CHANGED)

3 classes, 16 methods, 3 named. `Raid.findRandomSpawnPos`
draws from the level random in 1.21.11 (the raid's own random
in 26.1) — a real divergence: `find_spawn_pos` now takes a
`SpawnRandom` (the two draws it makes) threaded through
`Raid::tick` / `Raids::tick` from `game/raids.rs::tick_raids`,
which takes the level random out of `self.world.random` with
the patrol spawner's `mem::replace` dance. The raid fixture
passes a `NeutralRandom` (nextInt 0, nextFloat 0.75) because
the oracle holds the stand-in level's random neutral in raid
mode. Wave composition and the horn seed still draw from the
raid's own random (no javap hunks). The boss event lost its
UUID in 1.21.11 (the Rust never had one — no edit); the
ominous banner is built directly by `getOminousBannerInstance`
(same 8 layers, already in `OMINOUS_LAYERS` — citations
reworded in item_sync.rs, lan/guest.rs, item_data.rs);
`RegistryDataLoader$RegistryData` trades `validator` for
`requiredNonEmpty` (no Rust mirror); DataComponentMatchers is
parameter-name noise only. Tests: `raid::tests::` 13 passed
(horizon-client-lib); client_probe check clean.

## Item 144 — widely named: nbt (rank 2, NO-CHANGE)

29 classes, 46 methods, 0 named. The delta is visibility
(`private` -> package-private on `CompoundTag.readNamedTagData`,
`NbtOps$GenericListCollector`, `SnbtGrammar` fields/methods and
inners) plus compiler artifacts (`Objects.requireNonNull`
removals, a field-init order swap, `checkcast Object` in the
float-convert switch) and one new default method,
`ListCollector.acceptAll(Stream)` delegating to
`acceptAll(Iterable)` (no Rust mirror). The other 23 classes
have zero javap hunks despite the report's changed-method
counts — the item-73/91 decompile-noise pattern. Tests:
`nbt::` 14 passed, `snbt::` 6 passed, `nbt_path::` 12 passed.

## Item 145 — game/leads.rs (rank 2, CHANGED)

4 classes, 15 methods, 3 named. `LeashKnotRenderer`'s
KNOT_LOCATION moved in 1.21.11 from
`textures/entity/lead_knot/lead_knot.png` to
`textures/entity/lead_knot.png` (jar-verified both ways) —
the Rust's sprite lookup found nothing in the 1.21.11 atlas,
so the knot silently never drew; the lookup is now
`entity/lead_knot`. `getOrCreateKnot` inlines the 26.1
`getKnot` / `createKnot` split in 1.21.11 (same behaviour;
citation reworded). `interact` drops its `Vec3` parameter
(body identical); `renderSolid` -> `render`, `Matrix4fc` ->
`Matrix4f`, `LightCoordsUtil.pack` -> `LightTexture.pack`;
`FenceBlock` water check is the same predicate by type.
Tests: `game::leads::` 6 passed, 1 ignored.

## Item 146 — game/piston_push.rs (rank 2, NO-CHANGE)

1 class, 5 methods, 4 named. The whole delta is five
`BlockState.is(Object)` -> `BlockState.is(Block)` overload
changes (sticky piston, slime, honey, moving piston x2);
`finalTick`, `getCollisionRelatedBlockState`,
`moveCollidedEntities` and `tick` are otherwise
bytecode-identical, and the literal comparison returns zero
differences. Tests: `game::piston_push::` 2 passed, 7 ignored.

## Item 147 — lan/advancements.rs / cooking recipes (rank 2, CHANGED)

7 classes, 35 methods, 1 named. The 26.1 wrapper types are
gone in 1.21.11: `CookingBookInfo`, `Recipe$CommonInfo` and
`ItemStackTemplate` — cooking recipes take a plain group
string, category and `ItemStack`, and the codecs moved to
`RecipeSerializer` with per-type `cookingtime` defaults
(smelting 200, blasting/smoking/campfire 100, verified from
the 1.21.11 jar). The Rust defaulted every kind to 200 — now
per-kind — and the cooking result is forced to count 1
(`STRICT_SINGLE_ITEM_CODEC` has no count field). The eight
`ItemStackTemplate` citations in crafting.rs are reworded to
the 1.21.11 codec names. Recorded, not fixed: the loader's
lenient bare-id result read (0 vanilla uses) and
`SlotDisplay::ItemStack`'s count-sensitive derived `PartialEq`
(vanilla's 1.21.11 `equals` uses `ItemStack.matches`;
production never compares displays). Tests: `crafting::` 25
passed (1 new), `lan::advancements::` 13 passed.

## Item 148 — mobs/rabbit.rs (rank 2, CHANGED)

3 classes, 15 methods, 3 named. The hop lasts 10 ticks in
1.21.11 (15 in 26.1, both clocks); the fast landing delay is
1 tick (3 in 26.1); the hop's forward push is flat — no
vertical component (26.1 pushed up 0.5 for a baby, 1.5 for an
adult); and there are no baby dimensions — babies scale the
adult box by 0.5 (`Baby::Abs(0.24, 0.4, 0.39)` ->
`Baby::Scale(0.5, None)`). The animation states, their
`baseTick` driver and the `setLeashData` override are
26.1-only (the Rust's hop animation is procedural; the idle
clock was already removed). The farmland raid-target check
was already fixed in item 27. Tests: `mobs::rabbit::` 11
passed (1 new).

## Item 149 — datafix/files.rs (rank 2, CHANGED, citations only)

17 classes, 25 methods, 2 named. The whole `filefix` system
(16 classes, 78 jar entries in 26.1) is absent in 1.21.11 —
the module doc and layout code already say so, no behaviour
change. `writeStateProperties` is inlined into
`writeBlockState`/`writeFluidState` with identical output
(the template.rs mirror already matches the 1.21.11 form);
the one-argument `getDataVersion` is gone (the Rust takes a
default, matching the surviving two-argument form);
`addCurrentDataVersion` and the one-argument `prettyPrint`
are new in 1.21.11. Citations fixed: `canceled()` no longer
cites the 26.1-only `CanceledFileFixException` (1.21.11's
`WorldUpgrader.cancel` just stops the worker by flag),
`data_version` cites the two-argument form, and the
`Progress` doc says "file upgrades". Tests:
`datafix::files::` 5 passed.

## Item 150 — widely named: world/level/levelgen (rank 2, NO-CHANGE)

34 classes, 45 methods, 0 named. The `noNewCaves` refactor
gives the nether, caves and floating-islands routers shifted
overworld temperature/vegetation in 1.21.11 (26.1: zero
climate for caves/floating-islands, unshifted nether noises
for the nether) — covered by the 1.21.11 data-pack JSONs,
which the Rust reads via `NoiseRouter::from_json`; no Rust
mirror of NoiseRouterData's bootstrap exists. The
`WorldOptions` codec field is `generate_features` in 1.21.11
(`generate_structures` in 26.1) — `save.rs` already reads
and writes the 1.21.11 form. `NoiseChunk.fullNoiseDensity` /
`getInterpolatedDensity()` are 26.1-only with zero Rust
citations. The literal comparison across all 34 classes
returns exactly the one field-name difference. Tests:
`worldgen::router::` 2 passed.

## Item 151 — game/functions.rs (rank 2, CHANGED)

11 classes, 34 methods, 1 named. In 1.21.11 the `/schedule`
queue is not a SavedData at all: `PrimaryLevelData` holds
the `TimerQueue` and stores it in level.dat's
`Data.ScheduledEvents` as `{Name, TriggerTime, Callback}`
compounds, with the callback's function id under `Name`
(the 26.1 codec form was `trigger_time` / `id` /
`callback: {type, id}` in a separate
`data/minecraft/scheduled_events.dat`). The Rust's schedule
persistence was entirely 26.1-shaped and is now
1.21.11-shaped: `load_schedule` reads level.dat's
`ScheduledEvents` (unparseable callback drops the event,
`Name` defaults to `""`, `TriggerTime` to 0);
`save_schedule` builds the 1.21.11 list and hands it to the
level (`Level.scheduled_events`, a new field written into
`Data` by `write_internal` when set). Tests:
`game::functions::tests::scheduled_events` 2 passed (both
new; the first run was OOM-killed at the lib-test link, the
retry passed).

## Item 152 — mobs/turtle.rs (rank 2, CHANGED)

7 classes, 14 methods, 3 named. 1.21.11 has one concrete
`TurtleModel` (26.1 split it into Adult/Baby): its
`setupAnim` shows the egg belly whenever the turtle carries
an egg and sinks the root one unit, with no baby gate — the
Rust's `!s.baby` gate is dropped (the item-69 referral;
unobservable in play, only a bred adult carries an egg).
The baby transform lives in the prepared layer data. The
entity and goals are unchanged (`getAgeScale` is 0.3 for
babies in both versions, so `Baby::Scale(0.3, None)` stands;
the lay-egg goal reads the same level random by field).
Tests: `mobs::body_anim::` 8 passed (1 new).

## Item 153 — mobs/warden.rs (rank 2, CHANGED)

6 classes, 14 methods, 3 named. `Sensor.randomlyDelayStart`
is 26.1-only: in 1.21.11 the `Sensor` constructor itself draws
each sensor's first-scan delay from a static thread-safe
`Sensor.RANDOM`, never the entity's random. The Rust mirrored
the 26.1 form in all four brain files, so every brain init
consumed draws from the mob's random and shifted its later
draws against the oracle. All four now draw the delays from
the brain seed (`warden_brain::delay_sensors` takes no random
at all now — a type-level fact; animal/monster/villager brains
draw from `LegacyRandom::new(shuffle_seed)`), which keeps the
phases deterministic in harnesses. `Warden.BRAIN_PROVIDER` is
also 26.1-only (`makeBrain(Dynamic)` → `WardenAi.makeBrain`;
the warden.rs comment reworded, warden_brain.rs already done
in item 121). The warden's path maluses already use the
1.21.11 names (`DamageOther` 8.0, `DamageFire`/`DangerFire`
0.0); `canTargetEntity`'s `getType() ==` form is the same
predicate as the Rust's species comparison (armor stands are
not modelled as mobs here). `WardenModel` is bytecode-identical.
Tests: `mobs::warden::` 8 passed, `mobs::warden_brain::` 3
passed (1 new: the phases come from the brain seed), 2
ignored, `mobs::animal_brain::` 6 passed,
`mobs::villager_brain::` 2 passed, 4 ignored,
`mobs::breeze::` 7 passed, `mobs::creaking::` 5 passed.
Recorded, not fixed: `monster_brain.rs:1-2,99` and
`villager_brain.rs:1,312` still cite `BRAIN_PROVIDER` — their
own items' scope.

## Item 154 — widely named: world/level/block/entity (rank 2, CHANGED, citations only)

25 classes, 44 methods, 0 named. Mostly noise: the eleven `$1`
anonymous classes differ only by the `requireNonNull` compiler
artifact, and five classes (`CopperGolemStatueBlockEntity`,
`CrafterBlockEntity`, `JigsawBlockEntity`, `SkullBlockEntity`,
`SpawnerBlockEntity`) are bytecode-identical — this discharges
item 110's `CrafterBlockEntity` referral. The bell's RAIDERS check
is `getType().is(tag)` in 1.21.11, which the Rust already mirrors
(the tag is checked against the type id). Test-instance padding
(`getPadding` / `getTestBoundingBox` / `getTestBounds`) is 26.1-only
and the Rust already uses the unpadded structure box everywhere.
The one behaviour delta is the test-instance `export`: 26.1 gates it
on `testTemplates()` ("Test structure exporting is disabled"), while
1.21.11 always converts the saved `.nbt` to SNBT under
`StructureUtils.testStructuresDir`. The engine has no
structure-template packer (`NbtUtils.structureToSnbt` packs the
template before printing), so the export stays a stub and the
`export_test` doc now says so honestly. The literal comparison
confirms the `export` strings are the only literal differences
across all 25 classes. `cargo check -p horizon-client --lib
--tests --no-default-features` clean.

## Item 155 — environment_attributes.rs (rank 2, CHANGED)

12 classes, 22 methods, 2 named. `AttributeType.toFloat` and
`AttributeTypes.INTEGER` are 26.1-only (no Rust mirror either way),
and three environment attributes are 26.1-only:
`visual/block_light_tint`, `visual/night_vision_color`,
`visual/ambient_light_color` — removed from the table (51 → 48).
They were the uniforms 26.1's `lightmap.fsh` read, and 1.21.11's
shader is a different algorithm, so `update_lightmap` is rewritten
to it: the hardcoded cubic block-light tint, the dimension's
ambient light as the mix factor (0 / 0.1 / 0.25), the end-flash
colour as the ambient colour, block factor flicker + 1.5 (26.1:
1.4), boss darkening and darkness only where the ambient factor is
0, night vision as a scale-up to white, and the two 0.75 mixes. The
per-pixel math is extracted as `Renderer::lightmap_pixel` with a
test pinning four hand-computed pixels. `Sky` loses the two tint
fields and gains `ambient_light` / `ambient_color`, set by the
frame code and `end_flash_sky` (the flash colour is not gated by
Hide Lightning Flashes, only the sky-factor addition is). The
turtle-egg default is 0.0 in 1.21.11 (0.002 in 26.1) — the day
timeline's track still supplies 0.002 where it runs, so only
timeline-less dimensions change (eggs never hatch there); the
table default and the `turtle_hatch_chance` fallback both fixed.
The environment fixture no longer emits the removed attributes.
Tests: `environment_attributes::` 1 passed (new),
`gfx::tests::lightmap` 1 passed (new), `environment::` 8 passed,
`environment_timeline::` 3 passed, `level::growth_tests::` 21
passed, `fog::` 16 passed. Recorded, not fixed: the table still
carries three 26.3-only entries (`natural_mob_spawns`,
`creature_world_gen_spawn_probability`, `straw_bed_rule`) that
exist in neither 26.1 nor 1.21.11 — not this item's delta (item
107 already fixed the `natural_mob_spawns` reader).

## Item 156 — game/dedicated/predicate_graph.rs (rank 2, NO-CHANGE)

6 classes, 21 methods, 2 named. `MappedRegistry.componentLookup`
is 26.1-only (1.21.11's `freeze()` skips building it),
`bindTags(Map)` becomes the single-tag `bindTag`, the loot
validation drops the `ValidationContextSource` wrapper and the
`Validatable` bound, `ValidationContext.forField*` are 26.1-only
and `setContextKeySet` is new in 1.21.11 — none of it has a Rust
mirror. The file's two citations check out against the 1.21.11
jar: "Unbound values in registry" is in `MappedRegistry`, and
`ValidationContext$RecursiveReferenceProblem` is byte-identical
between the versions. Tests: `game::dedicated::predicate_graph::`
3 passed.

## Item 157 — game/fishing.rs (rank 2, CHANGED)

2 classes, 11 methods, 3 named. The bobber texture moved:
`textures/entity/fishing/fishing_hook.png` (26.1) →
`textures/entity/fishing_hook.png` (1.21.11) — the Rust looked it
up at the old path and the lookup silently failed, so the bobber
never drew (the item-145 lead-knot failure mode). Fixed to
`entity/fishing_hook`. The render type also changed
(`entityCutoutCull` → `entityCutout`), but the Rust's entity
pipeline already uses no culling, so that is already the 1.21.11
form. The `getNearPlane(fov)` comment updated to the no-arg
1.21.11 form (the Rust computes the point itself from the FOV).
The `FishingHook` entity delta is only `is` overload changes.
Tests: `game::fishing::` 13 passed, 5 ignored.

## CI proof run 44850889695 (recorded below)

Dispatched 01:33:40 UTC Nov 21, 2026 on behaviour/1.21.11 @
d967ed0 (the run-44839260120 record commit); completed SUCCESS
03:00:29 UTC Nov 21, 2026 (1h27m). Profile 5/18/0 of 23 — same
five passing (block_properties, component_patch, food, lan,
resource_pack). Verdict FAILED: D1 fail (23 dead production
tags, down from 24 — item 128 removed the
`minecraft:nether_portal_frame` read), D2 pass (cargo check 0
errors / 0 warnings — the `unused_mut` fix held — protocol
tests 440/440), D3 fail (pin work-items.json lagged at 321/492
at dispatch), D4/D5 blocked (profile-stamp quirk), D6 deferred
(no playtest.md). Pin work-items.json updated to 330/492 at
this record.

## CI proof run 44839260120 (recorded below)

Dispatched 05:31 NPT Nov 21, 2026 on behaviour/1.21.11 @
3e19296 (item 126); completed SUCCESS 01:31 UTC Nov 21, 2026
(1h45m). Profile 5/18/0 of 23 — same five passing as run
44789092726 (block_properties, component_patch, food, lan,
resource_pack). Verdict FAILED: D1 fail (24 dead production
tags, unchanged), D2 fail (one `unused_mut` warning in
mesher/tint_tests.rs:311, counted twice across the two check
logs — fixed in the commit after this record), D3 fail (pin
work-items.json lagged at 280/492 at dispatch), D4/D5 blocked
(profile-stamp quirk), D6 deferred (no playtest.md). Protocol
tests 440/440 on CI.

## Item 158 — widely named: world/level/block (rank 2, CHANGED)

31 classes, 41 methods, 0 named. The item-59 `supports_*` audit
landing in this item: the Rust read four 26.1-only support tags that
resolve to empty sets in the 1.21.11 tree, so cactus could only
survive on cactus, wither roses and lily pads could never survive,
and farmland under a fence gate died. Fixed in both survival systems
(`level/survive.rs` and `block_survival.rs`): cactus → cactus or the
sand tag; lily pad → water below or ice/frosted ice below (verified
from the jar that `IceBlock` covers frosted ice too, while packed and
blue ice are plain `Block`); wither rose → dirt tag, farmland,
netherrack, soul sand or soul soil (new `SurvivalRule::WitherRose`,
on the `Vegetation` precedent); farmland → not solid, or a fence gate
or a moving piston, hardcoded (the moisture tick still reads the
`maintains_farmland` tag — correct in both versions; only
`canSurvive` diverges, because 26.1's tag data extends it with the
two blocks and 1.21.11's holds only crops). The binding tables had
the same disease one level up: `FarmBlock` was keyed as
`FarmlandBlock`, so farmland bound NO prepare-time survival rule at
all (fixed in both the Rust prepare table and the Python parity
table, which also still keyed the lily pad as `LilyPadBlock`). The
`Blocks` `<clinit>` (21k lines) holds only: two 26.1-only blocks
(golden dandelion + potted), the `postProcess` →
`hasPostProcess` mechanism change (soul sand gains it in 26.1; the
Rust mirrors neither form), the FungusBlock split (item 757), the
stem-tag ctors (already ported), and diff-misalignment cascades of
the colour-variant registrations. The 26.1 `HangingSignBlock` /
`PlainSignBlock` "classes" are 26.1-only render interfaces with no
Rust surface. Tests: prepare `survival::tests::` 4 passed (2 new);
4 new level tests (early-return locally, run on CI); tag audit run
locally still 23 dead — the `supports_*` literals were never audit
positions, so D1 does not move on this item.

## Item 159 — crates/horizon-client/src/flatgen.rs (rank 2, NO-CHANGE)

3 classes, 10 methods, 3 named. No behaviour delta anywhere:
`ChunkGenerator` differs only by the `getTypeNameForDataFixer`
return type (no Rust mirror), `ChunkPos.x()`/`z()` method calls →
field reads, and the `ChunkPos.pack` → `asLong` rename;
`VoidStartPlatformFeature` is byte-identical apart from
`ChunkPos.containing` → the `ChunkPos(BlockPos)` ctor, and the Rust
mirror already matches 1.21.11 exactly (`PLATFORM_OFFSET` verified
from the jar's `<clinit>` as (8, 3, 8)); `WorldDimensionData`
differs only by ctor visibility. A javap-regex literal scan first
flagged 26.1-only strings, but a direct constant-pool parse shows
them in both pools — the only real CP differences are debug
artefacts (26.1's local-variable names vs 1.21.11's `$$N` names,
lambda numbering). Tests: `flatgen::` 4 passed
(horizon-client-lib; the horizon-client lib binary runs 0 with that
filter).

## Item 160 — game/commands/extra.rs (rank 2, CHANGED)

4 classes, 20 methods, 2 named. Two real deltas. (1) The
`/fetchprofile entity` subcommand is 26.1-only (with
`printForEntity`/`printForAvatar` and the `no_profile` error):
removed from the command tree (tree_extra.rs) and from
`cmd_fetchprofile`, whose fallthrough is now unreachable — 1.21.11
has only `name` and `id`. (2) The NBT text component's `plain` flag
and rich-styled rendering are 26.1-only: 1.21.11's `NbtContents`
has no `plain` field and its non-interpret path is `asString`
(string tag → raw value, else compact SNBT). `resolve_nbt_component`
now uses a `nbt_as_string` helper with that form (unit-tested) and
ignores `plain`; the stale `CompilableString` citation in the
selector doc fixed to `SelectorPattern.CODEC`. No delta:
`ScoreContents`/`SelectorContents` JSON shapes are identical,
`ObjectContents.fallback` was already ported, and
`ServerWaypointManager` has zero javap hunks (lambda renumbering
only). Recorded, not fixed: the Rust ignores custom `separator`s on
NBT/selector components (pre-existing gap, same in both versions).
Tests: text_commands 4 passed (1 new), fetchprofile 2 passed,
horizon-command 52 passed.

## Item 161 — game/machines.rs (rank 2, NO-CHANGE)

5 classes, 10 methods, 3 named. No behaviour delta: the two
screens differ only by the 26.1 `extractBackground(GuiGraphicsExtractor)`
vs 1.21.11 `render` + `renderBg(GuiGraphics)` split (item 87's
sweep owns the type rename; geometry and textures verified
identical — hopper 176×133, label at height-94, dispenser 176×166);
the composter check is an `is` overload change; dropper and hopper
differ only by `getRandom()` vs the `random` field, the same source
(the Rust uses `world.random` in both spots). Tests:
`game::machines::` 24 passed, 5 ignored.

## Item 162 — recipe_book_screen.rs (rank 2, CHANGED, citations only)

4 classes, 9 methods, 3 named. The whole diff is the 26.1
render-extraction rename (`extractRenderState`/`extractTooltip` on
`GuiGraphicsExtractor` → `render`/`renderTooltip` on `GuiGraphics`;
`text` → `drawString`): no behaviour delta, geometry and sprites
identical. The Rust already used the 1.21.11 names in its docs; the
one stale citation (`GuiGraphicsExtractor.item` etc. in the
`ItemCall` doc) is fixed to `renderItem`/`renderFakeItem`/
`renderItemDecorations`, verified from the 1.21.11 jar. The rest of
the tree's Extractor citations stay with item 87's sweep. Tests:
`recipe_book::` 23 passed.

## Item 163 — game/frame_geometry.rs (rank 2, NO-CHANGE)

6 classes, 38 methods, 0 named. The whole diff is 26.1's
model-baking API rework (`SpriteId`/`MaterialBaker`/`Interner`/
`dispatch` package vs `Material`/`SpriteGetter`/`PartCache`/`model`
package): no behaviour delta in any baked output. The 26.1-only
pieces (missing fluid model, `getBlockStateModel` helper,
`InternerImpl`, the identity-matrix bake arg) have no Rust mirror;
the 1.21.11-only material constants were already ported (banner /
shield bases) or are name lookups (fluids, fire). Recorded:
`DESTROY_STAGES_OIT` exists in neither version's ModelBakery and
its sprites in neither jar — upstream-Horizon behaviour, not a
vanilla delta. Tests: `model_library::` 2 passed.

## Item 164 — worldgen/live.rs (rank 2, NO-CHANGE)

3 classes, 8 methods, 3 named. The one real delta — 26.1's
`getBoneMealFeatures` (tag-filtered) vs 1.21.11's
`getFlowerFeatures` (flower-type-filtered) — was already ported in
item 27: `LiveFeatures.flower_features` walks the biome steps in
vanilla order and keeps `minecraft:flower` features. The rest is
API shape (holders vs values, `getSubFeatures` vs `getFeatures`,
empty-set construction). Recorded: the name-based list cannot
represent an inline configured flower feature, which 1.21.11's
type filter would include — impossible on vanilla data (verified:
no inline placed features in the 1.21.11 data pack). Tests:
bonemeal/grass filters pass.

## Item 165 — game/commands/data.rs (rank 2, NO-CHANGE)

4 classes, 17 methods, 2 named. No behaviour delta: `TagCommand`
is an `entityTags()` → `getTags()` rename; the data accessors
differ only by visibility; `PersistentEntitySectionManager`
differs only by visibility and the `ChunkPos.pack` → `toLong` /
`asLong` / `new ChunkPos(long)` renames, with identical
`processPendingLoads` logic. The Rust's uniform `pack`/`unpack`
naming across BlockPos/SectionPos/ChunkPos stays (1.21.11's Java
names are inconsistent across the three types; renaming would be
tree-wide churn for nothing). Tests: `game::commands::data::` 12
ignored (fixture-gated). Disk note: the first test run hit quota
exceeded; cleaned `target/debug/incremental` (24G), 103G → 93G.

## Item 166 — mobs/sniffer.rs (rank 2, CHANGED)

4 classes, 17 methods, 2 named. The baby sniffer is the delta:
26.1 gives babies a separate SniffletModel + snifflet.png texture
and never runs BABY_TRANSFORM; 1.21.11 uses the same model with
the BABY_TRANSFORMER-scaled baby layer, the adult texture, and
`babyTransform.applyStatic()` in setupAnim (head ×1.2, + (0, 1,
1)). The texture, the 0.5 baby scale and the baby layer were
already the 1.21.11 form; the animation was not — `anim` now
applies BABY_TRANSFORM at t=0 for a baby, last, as vanilla does.
The `BABY_START_AGE` doc fixed to the 1.21.11 `setBaby` form (the
-48000 value is the same). No delta: the powder-snow path type,
the brain sensors, the digging dimensions and the random source
all already match. Tests: `mobs::sniffer::` 3 passed (2 new).

## Item 167 — mobs/witch.rs (rank 2, CHANGED)

2 classes, 7 methods, 3 named. Two real deltas: the witch texture
moved (`textures/entity/witch/witch.png` in 26.1 →
`textures/entity/witch.png` in 1.21.11 — verified in both jars'
asset lists; the Rust had the 26.1 path, so the witch never drew),
and the potion throw is always speed 0.75 in 1.21.11 (26.1 slowed
close-range throws to 0.45 within 2 blocks). The potion selection,
aim and throw sound already matched. Tests: `mobs::witch::` 4
passed (1 new: the flat texture).

## Item 168 — debug_entries.rs (rank 2, CHANGED)

5 classes, 16 methods, 2 named. The debug-entries registry is the
delta: 1.21.11 registers 39 entries, not the Rust's 45 (26.3 form).
Gone: `detailed_memory`, the split `looking_at_*_state`/`_tags`
halves, `looking_at_entity_tags`, `player_speed` (a 26.2+
addition) and `day_count`; the merged `looking_at_block` /
`looking_at_fluid` take their place, and `post_effects` is
`post_effect` (singular). The F3 lines follow: the merged entries
draw state lines then tag lines in one group, as 1.21.11's
`DebugEntryLookingAtBlock`/`Fluid` do, and `looking_at_entity`
shows only the type (1.21.11 has no entity-tags entry). The
`debug-profile.json` upgrade chain ends at 4671: the 4763
looking-at split and 5001 post-effect rename postdate 1.21.11 and
no longer run. No delta: `load` / `rebuildCurrentList` / profiles
/ status names are identical in both versions (refactors and
renames only). Tests: `debug_entries::` 6 passed (1 new: the
registry is the 1.21.11 form); `cargo check -p horizon-client`
clean.

## Item 169 — game/llamas.rs (rank 2, CHANGED)

2 classes, 6 methods, 3 named. The baby passenger seats are
the delta: the Rust gave a baby chested horse or llama the
adult seat halved, which matches neither version. 1.21.11's
`BABY_DIMENSIONS` override the seat before the 0.5 scale — a
baby chested horse sits at its height less 0.15625, centred
(donkey 0.671875, mule 0.721875 after halving), a baby llama
at its height less 0.8125 (0.52875, z -0.15). Heights verified
from the 1.21.11 jar (donkey 1.5, mule 1.6, llama 1.87). No
delta: the feeding values, the baby age-up without an age-lock
check, the interact order and the breeding roll already match
1.21.11; 26.1's golden-dandelion exception is 26.x-only (the
item does not exist here). Tests: `mobs::riders::` 13 passed
(1 new: the baby seats).

## Item 170 — game/natural_spawning.rs (rank 2, NO-CHANGE)

1 class, 16 methods, 2 named. Pure renames: `SavedDataStorage`
→ `DimensionDataStorage`, `ChunkPos.pack` → `asLong`/`toLong`,
`ServerLevel.getRandom()` → the `random` field, and two fields
widened to package-private. `tickChunks` / `tickSpawningChunk`
are behaviourally identical — the Rust's `spawn_natural`
already implements the shuffle, the 400-tick creature gate
and the per-chunk gate. No code changed.

## Item 171 — command/item.rs (rank 2, CHANGED)

1.21.11's `ItemParser.parse` validates the parsed stack
(`ItemStack.validateComponents` on the patched map) and 26.1
does not. Ported: a damageable item cannot stack ("Item cannot
be both damageable and stackable"), and a container's stacks
cannot exceed their maximum ("Item stack with count of {n} was
larger than maximum: {m}"), both as `arguments.item.malformed`
with the cursor at the end of the argument. The parser had no
per-item defaults, so the oracle's `registries` export now
carries `item_defaults` (each item's default `max_stack_size`,
plus `max_damage` when damageable), read into the new
`Registries.item_defaults` by `fill_from_export` /
`fill_from_exported` and the parse fixture; an item the export
does not list skips the check. The predicate grammar,
`IdentifierParseRule` and the parser's state machine are
unchanged (name noise + access widening only). Corpus gains 6
valid + 4 invalid cases. Gaps (pre-existing): the component
codec does not decode `container` at all, so codec-level
container failures still parse; a container entry's patch
removing `max_stack_size` is not honoured.

## Item 173 — level/interact.rs (rank 2, CHANGED)

`DragonEggBlock.teleport`: 1.21.11 drops 26.1's requirement
that the teleport target have a non-air block below it — the
egg now teleports to the first air block inside the build
height and falls from there. `ComposterBlock` is unchanged
(name noise, access widening and the `getRandom()` →
`random` rename only; the compostables table diffs clean).
The sim still has no world-border check on the teleport, a
pre-existing gap in both versions.

## Item 174 — worldgen/structure/legacy.rs (rank 2, NO-CHANGE)

2 classes, 4 methods, 3 named. No behavioural delta: six
`BlockState.is(...)` calls resolve to the `is(Block)` overload
in 1.21.11 where 26.1 called `is(Object)` (same block-identity
test in both), and `TemplateStructurePiece`'s `template` /
`placeSettings` fields lose `final`. `createChest`,
`createDispenser`, `reorient` and the replaceability check are
otherwise identical. No code changed.

## Item 175 — gfx/pack_atlas.rs (rank 2, CHANGED)

`SpriteLoader.stitch`: 1.21.11 counts anisotropy only with
mipmaps (`mipLevel != 0`), where 26.1 counts it whenever
filtering is anisotropic — at mip level 0 the atlas padding is
1 px instead of `1 << (bit - 1)`. Ported in
`PackAtlas::stitched`. The rest of `stitch` is identical.
`ChunkSectionLayer` / `SectionCompiler` have no portable delta:
1.21.11 adds a `TRIPWIRE` layer, but the Rust's mesh has two
buckets (`solid`, `translucent`) and already folds tripwire
into translucent; the leaves fancy/fast switch and translucent
sorting are already modeled, and the buffer sizes, vertex
formats and per-block crash report are GL-side details the
wgpu renderer does not model.

## Item 176 — hud_overlays.rs (rank 2, NO-CHANGE)

6 classes, 13 methods, 2 named. No behavioural delta:
`BossHealthOverlay` / `SubtitleOverlay` only rename
`extractRenderState(GuiGraphicsExtractor)` to
`render(GuiGraphics)` (the extractor does not exist in
1.21.11) with every layout number identical;
`DeltaTracker.Timer` fuses its two advances into
`advanceTime(long, boolean)` with identical bodies;
`MobEffectInstance` is bytecode-identical. Doc citations in
`hud_overlays.rs` updated to the 1.21.11 method names. The
`GuiGraphicsExtractor.itemDecorations` citations there belong
to item 371, which owns the extractor rewire.

## Item 177 — game/beacon.rs (rank 2, NO-CHANGE)

2 classes, 23 methods, 1 named. No behavioural delta:
`BeaconBlockEntity.tick` reads `getLightBlock()` where 26.1
reads `getLightDampening()` — verified a pure rename by javap
on both jars (identical computation bodies); the rest of the
tick, `updateBase` and `applyEffects` is identical.
`BeaconScreen` only renames the render-state-extraction
methods to the `render*` forms with every layout number
identical. No code changed. (The Rust's block-facts field
keeps the 26.1 name `dampening` for the same value.)

## Item 178 — level/shapes.rs + tick.rs + survive.rs + block_survival.rs (rank 2, CHANGED)

3 classes, 13 methods, 2 named. Real delta: 1.21.11 hardcodes
what 26.1 reads from three bubble-column tags that do not
exist in 1.21.11 (`bubble_column_can_occupy`,
`enables_bubble_column_push_up`, `enables_bubble_column_drag_down`
— each held only its one block/fluid in 26.1's data). Missing
tags resolve to empty sets in the Rust, so the tag-based code
would never extend a column into water and would fail survival
above soul sand/magma. Ported: `canOccupy` -> `canExistIn`
(bubble column, or source water with amount >= 8);
`canSurvive`/`getColumnState` check soul sand/magma directly;
`updateColumn` drops its block parameter and returns water
when the below block supports neither; the survival oracle's
`bubble_up`/`bubble_down` sets are the hardcoded blocks.
Coral placement (`isFull()` == `getAmount() == 8`) and stair
placement (`is(WATER)` == `getType() == WATER`) are pure
renames — no change. Tests: 3 new level tests (push-up,
drag-down, unsupported -> water); they skip locally for
missing prepared resources and run in CI.

## Item 179 — worldgen/tree.rs (rank 2, NO-CHANGE)

4 classes, 23 methods, 1 named. No behavioural delta:
`WorldGenLevel` parameters widen to `LevelSimulatedReader`
in `FoliagePlacer` and `TreeFeature.getMaxFreeTreeHeight`;
`tryPlaceLeaf` calls `foliageProvider.getState(random, pos)`
instead of `getState(level, random, pos)` — no vanilla
provider reads the world, and the Rust already carries that
delta in `state_provider.rs`; `IntProviders.codec` moved to
`IntProvider.codec` (same codec); `PlacementContext` is pure
renames. No code changed.

## Item 180 — gamerules.rs (rank 2, NO-CHANGE)

2 classes, 12 methods, 2 named. No behavioural delta: the
rule registry is identical in both versions (59 rules,
verified name-for-name against the Rust table); the
1.21.11 `GameRules` constructor (fresh defaults + overlay
of available rules) is already what `copy_with_features`
and `from_nbt_with_features` implement; `set` throwing on
an invalid rule (vs 26.1's warn-and-ignore) is unreachable
on every path that matters and the Rust's bool-returning
`set` is the idiomatic equivalent; the 26.1 `SavedData`
superclass, `reset`, and the `List` constructor have no
Rust analogue. No code changed.

## Item 181 — pack_screen.rs (rank 2, NO-CHANGE)

5 classes, 22 methods, 1 named. No behavioural delta:
`PackSelectionScreen` is renames + access widening with
identical layout numbers; `TransferableSelectionList` is
the render-state-extraction rename with identical layout
numbers; `scrollbarWidth()` -> literal 6 and `scrollable()`
-> `scrollbarVisible()` are verified equivalent by javap
on `AbstractScrollArea` (26.1's default width is 6, both
predicates are `maxScrollAmount() > 0`). One doc citation
updated (`Screen.extractRenderState` -> `Screen.render`).

## Item 182 — mobs/camel.rs (rank 2, CHANGED)

2 classes, 12 methods, 2 named. Real delta: `Camel.BABY_SCALE`
is 0.45 in 1.21.11 (26.1: 0.6) — it reaches the seat and the
lead via `LivingEntity.getPassengerRidingPosition` (scale =
`getScale() * getAgeScale()`, verified by javap) and
`getLeashOffset`. Ported: a `BABY_SCALE` const (0.45) now
feeds `leash_offset` and `passenger_attachment_at` (both
hardcoded 0.6). No delta: `FoodOnAStickItem.use` (the type
test is the same either way), the golden-dandelion
`mobInteract` branch (the item does not exist in this tree),
`handleEating`'s `canAgeUp()` vs `isBaby()` (no age lock in
the Rust), the Brain API rework (the Rust models camel AI
with goals). Test: a baby camel seats/leashes at the 0.45
scale (9 passed in `mobs::camel::`).

## Item 183 — game/tnt.rs (rank 2, NO-CHANGE)

2 classes, 11 methods, 2 named. No behavioural delta:
`PrimedTnt.tick` calls `updateFluidInteraction()` in 26.1
and `updateInWaterStateAndDoFluidPushing()` in 1.21.11 —
the two versions' forms of the same step (verified by
javap; the Rust's shared `push_by_fluid` already implements
it and the TNT tick already calls it in the same place);
the constructor's `getRandom()` -> `random` field is the
same random; `TntRenderer` only changes how the block state
reaches the render state (resolver vs direct assignment)
while the fuse/swell/flash math the Rust implements is
identical in both versions. No code changed.

## Item 184 — game/consume.rs (rank 2, NO-CHANGE)

2 classes, 10 methods, 2 named. No behavioural delta:
`UseRemainder.convertIntoRemainder` is identical in both
versions — the `ItemStackTemplate` vs `ItemStack` change
is only how the remainder stack is produced, and the Rust
abstracts that behind `use_remainder::convert`'s `create`
callback fed by an `ItemTemplate` (the same JSON shape);
`TeleportRandomlyConsumeEffect.apply` is identical except
that 1.21.11 resets the impulse context only for players,
which is already the Rust's behaviour (the shared
`TeleportConsume::apply` runs only for the local player;
the fox has its own implementation with no impulse reset).
No code changed.

## Item 185 — block_entity_anim.rs (rank 2, NO-CHANGE)

3 classes, 19 methods, 1 named. No behavioural delta:
`ChestModel`'s double-chest face culling (26.1 only) is
invisible (hidden faces) and the geometry comes from the
version's `entity-models.json` export anyway; the
`entityCutoutCull` -> `entitySolid` change is invisible
for the opaque chest texture; `DecoratedPotRenderer`'s
wobble math and model transformation are identical in
both versions and the Rust matches them term-for-term
(the sprite/material rework is renderer infrastructure);
`BlockTags`' removed/added tags match the 1.21.11 data
files exactly (verified against both jars) and the Rust
loads its tags from that data. NOTE: live reads of
removed `supports_*` tags remain in survival/placement
code — they belong to their owning plant classes, which
are rank-3 items (e.g. `SugarCaneBlock` is not in the
rank-2 triage); the `data.rs` fields stay until those
items convert the reads. No code changed.

## Item 186 — mobs/zombie_villager.rs (rank 2, CHANGED)

1 class, 9 methods, 2 named. Real delta: 1.21.11 drops
`VillagerDataFinalized` (26.1's synched flag + NBT tag;
the finalize gate is the spawn reason instead). The
finalize gate, baby dims and the regular Villager were
already ported; what was left was the zombie villager's
NBT: `write_nbt` no longer writes the flag, `read_nbt` no
longer reads it, and the now-dead `data_finalized` field
is removed from `villager::Ext`. No delta: `mobInteract`
(golden apple + weakness -> cure), `startConverting`,
cure progress, `finishConversion`. Test: a zombie
villager save has no finalized flag (7 passed in
`mobs::zombie_villager::`).

## Item 187 — gizmos.rs (rank 2, NO-CHANGE)

5 classes, 9 methods, 2 named. No behavioural delta: the
bounding-box renderer is pure renames; the chunk-border
renderer only changes how it reaches the camera entity
(the same entity either way); the hitbox renderer drops
26.1's frozen-aware per-entity partial tick for the
frame's partial — which is already what the Rust uses;
`Gizmos` is renames + access widening. No code changed.

## Item 188 — game/music.rs (rank 2, NO-CHANGE)

2 classes, 8 methods, 2 named. No behavioural delta:
`PauseScreen` is the render-state-extraction rename with
identical layout numbers and identical feedback/dialog
logic; `ServerReconfigScreen` is only the
`addRenderableWidget` return-type artifact. (26.1's
`OptionsScreen` takes a third `inWorld` constructor
argument; that belongs to the OptionsScreen item.) Doc
citations updated (`extractRenderState` -> `render` /
`renderToast`, `extractBackground` -> `renderBackground`).

## Item 189 — mobs/goals.rs (rank 2, NO-CHANGE)

7 classes, 8 methods, 2 named. No behavioural delta:
`BreathAirGoal.start` drops 26.1's `getNavigation().stop()`
after `findAirPosition()` — the Rust already matches
1.21.11 (it stops the old path, then moves to air, and
`nav_move_to` overwrites the path anyway); the powder-snow
and hurt-by goals only change how the same type test is
spelled; `EatBlockGoal` is identical;
`NearestAttackableTargetGoal` only loses a `final`; the
bee/polar-bear inner goals are renames + widening. Two doc
citations corrected (`ClimbDangerPowderSnowGoal` ->
`ClimbOnTopOfPowderSnowGoal`, the real class name).

## Item 190 — mobs/wither.rs (rank 2, NO-CHANGE)

3 classes, 8 methods, 2 named. No behavioural delta:
`WitherBossRenderer` is bytecode-identical; `WitherBoss`
(`aiStep`, `customServerAiStep`, `performRangedAttack`,
`hurtServer`, death loot, head tracking) is identical
apart from names and the `is(tag)` / `getType().is(tag)`
spelling; the boss-event construction difference (26.1
draws a UUID from the entity's random, 1.21.11 takes no
UUID) has no Rust analogue — the Rust models neither the
boss-bar UUID nor a random draw for it. No code changed.

## Item 191 — util (rank 2, NO-CHANGE)

17 classes, 28 methods, 0 named. No behavioural delta:
`ArrayListDeque` is bytecode-identical; `CubicSpline` /
`ProblemReporter` / `Util$OS` differ only by widening and
unboxing; `ExtraCodecs` loses 26.1-only path codecs (Java
plumbing, no Rust analogue); `SpawnUtil$Strategy` is
`is(Object)` -> `is(Block)` overload noise;
`LightCoordsUtil` is 26.1-only — its helpers live in
`LightTexture` / `LevelRenderer.getLightColor` /
`ModelBlockRenderer$AmbientOcclusionRenderStorage.blend`
in 1.21.11, and the Rust's mesher lighting already
implements those exact forms (both blends and the
light-at-position formula verified against the 1.21.11
jar). Doc citations updated to the 1.21.11 homes. No code
changed.

## Item 193 — game/bossbars.rs (rank 2, CHANGED)

2 classes, 17 methods, 1 named. Real delta:
`CustomBossEvents.create` takes no random in 1.21.11 —
`ServerBossEvent` draws the bar's UUID from the static
`Mth.RANDOM`, where 26.1 drew it from `level.random`.
`/bossbar add` no longer advances the level random: the
Rust now draws the fresh UUID from
`random_source::entity()`, the tnt.rs precedent for a
non-level random. The `SavedData` -> plain-class change
has no NBT effect (the id -> bar map codec is unchanged);
`BossBarCommands` is renames plus the
`ComponentUtils.updateForEntity` resolution API. New
`game/bossbars_tests.rs` asserts the bar gets a fresh UUID
and the level random's next draw is unchanged (runs in CI;
needs prepared resources).

## Item 194 — lan/chunks.rs (rank 2, NO-CHANGE)

1 class, 7 methods, 2 named. The protocol delta is
already ported by the version adapters, both wired:
`translate::v1_21_11::chunk` reads one short per section
and writes two (fluids: 0) for the 26.1-layout interior,
`client_session::step_1_21_11::chunk` drops the fluids
short going the other way; the internal `Section` keeps
`fluids` as the 26.1 form and the client goes by the
states. Known gap, NOT ported: 1.21.11's `setBlockState`
counts ANY non-empty fluid toward `tickingFluidCount`
(no random-tick gate — verified in bytecode), where
`recalcBlockCounts` keeps the gate, so a section that
gains water via setBlockState reports
`isRandomlyTicking()` in 1.21.11; the Rust's
`RandomTickSections` is content-derived (the recalc
form) and misses it. Observable only as random-draw
drift — the random-tick-sections parity harness measures
it, but that harness is not in the CI profile. The
faithful port needs a per-section load-time fluid
baseline (see rank2-item-194-TRIAGE.md); a half-port
with the baseline defaulting to 0 would regress every
loaded water section, so it is left for a later item
with the oracle harness to verify.

## Item 195 — text_display.rs (rank 2, NO-CHANGE)

3 classes, 7 methods, 2 named. No behavioural delta:
`Brightness` gains the static `pack`/`block`/`sky`
helpers in 1.21.11 (moved in from the 26.1-only
`LightCoordsUtil`, same formulas — the Rust already
cites `Brightness.pack`); `Display`/`BlockDisplay`/
`ItemDisplay` are renames; `TextDisplay`'s load-time
text resolution changes API (`resolve` with a
`ResolutionContext` -> `updateForEntity` with the entity
passed explicitly) with the same semantics, and the Rust
resolves display text at render time anyway. The Rust's
`text_display_data` matches vanilla field for field. No
code changed.

## Item 196 — level/rails.rs (rank 2, CHANGED)

2 classes, 7 methods, 2 named. Real delta:
`TripWireHookBlock.calculateState` in 26.1 bails after
the far hook's notification if the near hook vanished
(`onRemoved` + return, no state emitted); 1.21.11 drops
the check and emits unconditionally. Removed the
early-return from `tripwire_hook_calculate`; the far
`tripwire_emit` now always runs. `BaseRailBlock` is
overload noise only; `onRemove`'s `onRemoved` inline is
the same behaviour (the Rust keeps the shared helper).

## Item 197 — world/level (rank 2, CHANGED)

8 classes, 27 methods, 0 named. Two real deltas in
`LevelSettings.parse` (the Rust already read the
1.21.11 flat level.dat form): an out-of-range
`Difficulty` id WRAPS (`Difficulty.byId`, `ByIdMap`
WRAP — verified in the 1.21.11 jar) instead of falling
back to normal, and a missing `allowCommands` key
defaults to gameType == CREATIVE. Both ported in
`save.rs::read_root`, with a new unit test. The rest is
noise: `ClipContext$Block`'s fall-damage lambda is
identical apart from the `is`/`getType` spelling,
`LevelHeightAccessor` loses a 26.1-only overload,
`DifficultySettings`/`withDifficultyLock` are gone
(the lock lives in `DifficultyLocked`, already read).

## Item 198 — brewing.rs (rank 2, NO-CHANGE)

3 classes, 6 methods, 2 named. No behavioural delta:
`TippedArrowItem`/`RecipePropertySet` are a type
widening and a rename; `BrewingStandBlockEntity` is
widening, `is` overloads, and the `getCraftingRemainder`
template/stack distinction (same as item 184) with
identical slot-or-drop logic, which the Rust's `brew()`
already matches. No code changed.

## Item 199 — datapack.rs (rank 2, CHANGED — test fixtures only)

5 classes, 6 methods, 2 named. No behavioural delta in
vanilla: `MacroFunction.stringify`'s cases are identical
(javac codegen only); `Pack.readPackMetadata`'s fallback
is reached with the same result in both versions (the
fallback codec needs only a description — the Rust's
folder gate already requires exactly that); the other
two classes are widening/rename. Fixed three tests that
failed on the clean branch tip: their fixtures used
26.x pack formats (data 121, resource 97) while 1.21.11
is data (94, 1), resource (75, 0) — verified against
the jar's version.json. Fixtures updated in
`datapack.rs` (create-world install, folder-metadata)
and `packs.rs` (compatibility ranges, overlays). No
production code changed.

## Item 200 — mobs/parrot.rs (rank 2, NO-CHANGE)

3 classes, 6 methods, 2 named. No behavioural delta:
the PathType renames (`FIRE`/`FIRE_IN_NEIGHBOR` ->
`DAMAGE_FIRE`/`DANGER_FIRE`) are already the Rust's
names; `imitateNearbyMobs` is identical apart from the
order of the random load (same draws); the rest is
`is`/`getRandom` renames and widening. Noted: vanilla
picks from all non-parrot mobs and falls back to
`PARROT_AMBIENT` for unmapped types, where the Rust
filters to imitable species first — a pre-existing
divergence from both versions, not this item's delta.
No code changed.

## Item 201 — enchant.rs (rank 2, NO-CHANGE)

2 classes, 16 methods, 1 named. The one enchantment
delta (LUNGE's food/creative gate moving from the
enchantment data in 26.1 into `Player.lungeForwardMaybe`
in 1.21.11) is already the Rust's form
(`post_piercing_attack` gates on creative-or-food->=7);
all 43 enchantment definitions are otherwise identical.
`RegistryDataLoader`'s missing 1.21.11 registries
(sound variants, world clock, villager trade sets) are
already reflected — the data-driven trade system loads
empty and the hardcoded `VillagerTrades` port is item
751's work, as merchant.rs documents. No code changed.

## Item 202 — util/valueproviders (rank 2, NO-CHANGE)

3 classes, 26 methods, 0 named. The 26.1 interface /
1.21.11 abstract-class restructure of `IntProvider`
(and the `FloatProviders` factory folding into
`FloatProvider`) changes no sampling or codec
semantics: `UniformInt.sample` is
`Mth.randomBetweenInclusive` in both, and the Rust's
`worldgen/value_provider.rs` already implements the
1.21.11 codec forms (bare number = constant, dispatched
objects, max>=min validation) for all 6 registered int
and 4 registered float types, verified against the
1.21.11 jar. The Rust's extra `very_biased_to_bottom`
/ `trapezoid` int variants are 26.1-only but
unreachable on 1.21.11 data. No code changed.

## Item 203 — screen_draw.rs (rank 2, NO-CHANGE)

8 classes, 15 methods, 1 named. All render-state-
extraction renames with identical bodies:
`AbstractButton`'s `extract*` methods become
`render*` (`renderWidget`, `renderContents`,
`renderDefaultLabel`, `renderDefaultSprite`) on
`GuiGraphics`; the scissor stack moves from
`GuiGraphicsExtractor` to `GuiGraphics` itself (same
push/pop/peek API, verified in the 1.21.11 jar).
Four doc citations updated to the 1.21.11 names
(screen_draw.rs, title.rs); no behavioural code
changed. The remaining `extractWidgetRenderState`
citations (multiline_edit.rs, multiplayer.rs) belong
to their classes' own items / item 371.

## Item 204 — EntityType (rank 2, CHANGED)

1 class, 5 methods, 2 named. The registration table
diffs to exactly one real delta: the rabbit is
0.4 x 0.5 in 1.21.11 (26.1: 0.49 x 0.6, eye 0.59),
with the default eye height height * 0.85 = 0.425
(verified by javap on the 1.21.11 jar). Production
dimensions flow from the prepared `entity-types.json`,
so only the rabbit test fixture needed correcting
(eye height 0.4 -> 0.425; the baby test now asserts
the halved 0.2125). `create` and `isBlockDangerous`
differ only by the `getRandom()` field access and the
`is(Block)` overload spelling; 1.21.11 adds two `is`
convenience overloads.

## Item 205 — cartography.rs (rank 2, NO-CHANGE)

7 classes, 15 methods, 1 named. The screen diff is
pure render-extraction rename (`extractBackground` ->
`renderBg`, `extractResultingMap` -> `renderResultingMap`,
`extractMap` -> `renderMap`) with identical layout
numbers and error conditions; the menu diff is
parameter-name noise plus `lastSoundTime` widening —
slots, `onTake`, `slotsChanged`, `setupResultSlot`
(paper -> SCALE, glass pane -> LOCK, map -> x2) and
`quickMoveStack` are all identical. One doc citation
updated in `draw_cartography`; no behavioural code
changed.

## Item 207 — world/item (rank 2, CHANGED)

8 classes, 25 methods, 0 named. The one real delta in
the `Items` table: 1.21.11 lowers the spear dismount
condition's attacker-speed threshold (`KineticWeapon`'s
first `ofAttackerSpeed` argument) for every material
but wood — stone 13->10, copper 12->9, iron 11->8,
golden 13->10, diamond 10->7.5, netherite 9->7 blocks/s
(verified against the 1.21.11 jar's
`Item$Properties.spear`). Ported in
`spear.rs::Kinetic::of` with a test pinning all seven
thresholds. The rest is noise or data: `GOLDEN_DANDELION`
is 26.1-only (registry comes from prepared data);
eggs/dyes/nether star/shield/banner patterns/goat horn
change only `delayedComponent` construction, same
values; `ItemStackTemplate` is 26.1-only as a class but
the Rust already reads its `{id, count?, components?}`
shape directly; `MapItem` drops a null ItemFrame
argument; `ItemStack$1` renames `typeHolder` to
`getItemHolder`; `ProjectileWeaponItem` uses the
`is(Item)` overload.

## Item 208 — chunk.rs (rank 2, NO-CHANGE)

4 classes, 4 methods, 2 named. All noise:
`SerializableChunkData` reads `ChunkPos.x`/`z` as
fields instead of accessors and builds a `ChunkPos`
with the constructor instead of `unpack(long)`, with
the structure-reference validation (chessboard distance
<= 8) identical; `SectionData` and
`NeighborsUpdateRenderer$LastUpdate` widen record
access; `Heightmap` widens its predicate fields and
uses the `is(Block)` overload for the AIR test. No code
changed.

## Item 209 — tab_list.rs (rank 2, NO-CHANGE)

3 classes, 14 methods, 1 named. Pure render-state-
extraction rename with identical bodies:
`PlayerTabOverlay`'s `extractRenderState` /
`extractPingIcon` / `extractTablistScore` /
`extractTablistHearts` become `render` /
`renderPingIcon` / `renderTablistScore` /
`renderTablistHearts` on `GuiGraphics`
(`text` -> `drawString`); `PlayerFaceExtractor`
becomes `PlayerFaceRenderer.draw`. Four doc citations
updated in tab_list.rs; no behavioural code changed.

## Item 210 — screens.rs (rank 2, NO-CHANGE)

3 classes, 14 methods, 1 named. Render-extraction
renames with identical bodies (`WinScreen`'s
`extractRenderState` / `extractBackground` /
`extractVignette` / `extractMenuBackground` become
`render` / `renderBackground` / `renderVignette` /
`renderMenuBackground`; `SpriteIconButton`'s
`extractSprite` / `extractContents` become
`renderSprite` / `renderContents`). The poem random
is a real version delta — 26.1's
`createThreadLocalInstance(8124371L)` is Xoroshiro,
1.21.11's `create(8124371L)` is a LegacyRandomSource
(verified by javap on both jars) — but the Rust's
`add_poem_file` already uses `LegacyRandom`, the
1.21.11 form. Four doc citations updated; no
behavioural code changed.

## Item 211 — SweetBerryBushBlock (rank 2, CHANGED)

1 class, 4 methods, 2 named. The real delta is
`isValidBonemealTarget`: 26.1 requires age < 3 plus
air above inside the build height, 1.21.11 requires
only age < 3 — already the Rust's form in
`bonemeal.rs`, now pinned by a level test (a bush at
age 0 under solid stone still consumes bonemeal and
grows). `entityInside`'s fox/bee exemption and
`useItemOn`'s bone-meal pass differ only by the
`getType()` / `is(Item)` spelling and the 26.1-only
`ItemInstance` type name. No production code changed.

## Item 212 — onboarding.rs (rank 2, CHANGED)

2 classes, 14 methods, 1 named. The onboarding screen
differs: 26.1 shows four buttons in a 2x2 grid
(narrator, sounds, accessibility, language), 1.21.11
shows three in a single column (narrator, accessibility,
language — no sounds button). Ported: the layout is
now a column of three 150-wide buttons in 4 px cells,
the SOUNDS id and its click arm are removed, and the
layout test pins the new positions. Fade timings,
title, footer and logo are identical.
`WorldGenSettings` changes from a 26.1 `SavedData`
subclass to a 1.21.11 plain record with the same CODEC;
the Rust already writes the settings into level.dat's
`Data`, the 1.21.11 form — no change.

## Item 215 — BlockItem (rank 2, NO-CHANGE)

1 class, 3 methods, 2 named. `place` differs only by
the `is(Block)` overload and the 26.1-only
`ItemInstance` type name; `onDestroyed` spills the
same container contents via an Iterable instead of a
Stream. 1.21.11 adds `BlockItem.requiredFeatures()`
(delegating to the block), but the Rust models no
per-item/per-block feature-flag gating, so there is
nothing to port it into. No code changed.

## Item 216 — ambience.rs (rank 2, NO-CHANGE)

2 classes, 3 methods, 2 named. Pure noise: the bubble-
column test uses the `is(Block)` overload, and the
underwater handler reads the level's `random` field
instead of `getRandom()` — the same random, the same
draws. No code changed.

## Item 217 — wind.rs (rank 2, NO-CHANGE)

3 classes, 3 methods, 2 named. `WindChargeItem.use`
inlines 1.5f in 26.1 and reads the named
`PROJECTILE_SHOOT_POWER` (1.5f) in 1.21.11 — the same
value the Rust uses; `canHitEntity`'s end-crystal test
is the same under the `getType()` spelling.
`EntityRenderState.scoreText` is 26.1-only as a field,
but the below-name behaviour it carries exists in
1.21.11 (`ClientMannequin.belowNameDisplay()` is in
the jar) and the Rust models the behaviour, not the
field. No code changed.

## Item 218 — mobs/brain.rs (rank 2, NO-CHANGE)

6 classes, 23 methods, 0 named. The brain
serialization restructures (26.1's `Packed` record +
`ActivitySupplier` vs 1.21.11's direct `Codec<Brain>`)
but both write the same `Brain.memories` NBT
(`{value, ttl?}` per memory, verified in the 1.21.11
decompile); the outer `Brain` class is
bytecode-identical, so tick/activity logic is
unchanged. `Behavior.getRequiredMemories()` is a
26.1-only getter. Two doc citations in brain.rs
updated from `Brain.Packed` to the `memories` codec;
no behavioural code changed.

## Item 220 — level/growth.rs (rank 2, CHANGED)

2 classes, 3 methods, 2 named.
`TreeGrower.getMinimumHeight` is 26.1-only, so the
sapling and azalea bonemeal checks lose their
build-height gates: `SaplingBlock.isValidBonemealTarget`
is unconditionally true in 1.21.11 (verified by javap
on the jar) and `AzaleaBlock.isValidBonemealTarget`
asks only for the fluid above to be empty.
`tree_minimum_height` and the `TreeGrower.shortest`
field are removed as dead. `JigsawPlacement` is
identical in both versions (`$$N` noise only). New
level test: azalea at y 319 still takes bonemeal.

## Item 221 — level/weather.rs (rank 2, NO-CHANGE)

1 class, 3 methods, 2 named. `Biome.shouldFreeze` /
`shouldSnow` are logic-identical: the only changes are
visibility widening and the `is(Object)` →
`is(Block)` / `getType() == WATER` spelling of the
same tests. No code changed.

## Item 222 — worldgen/structure/end_city.rs (rank 2, NO-CHANGE)

1 class, 3 methods, 2 named. `EndCityPieces` differs
only by visibility widening (private → package-private
on the generator tables and `addPiece` / `addHelper` /
`recursiveChildren`); every method body is
bytecode-identical. No code changed.

## Item 223 — mobs/illusioner.rs (rank 2, NO-CHANGE)

4 classes, 12 methods, 1 named. `CameraRenderState`
moves package in 1.21.11; the `submit` bodies, the
illusion loop and the spell-goal logic are
bytecode-identical. The constructor diffs are dropped
null-check codegen plus visibility widening. No code
changed.

## Item 224 — environment_system.rs (rank 2, NO-CHANGE)

5 classes, 12 methods, 1 named. The 26.x world-clock
system (`ClockManager`, `Timeline$TimeMarkerInfo`,
the timeline's `clock` field and `time_markers`) is
absent in 1.21.11: timelines sample at
`level.getDayTime()` directly. `Timelines.OVERWORLD_DAY`
is renamed `DAY` (same key); every track keyframe is
identical, and `ValueSampler` is logic-identical. The
Rust already implements the 1.21.11 form (`Timeline`
with `period` + `tracks` only, day-time sampling, the
codec-shape test). No code changed.

## Item 225 — level/redstone.rs (rank 2, NO-CHANGE)

2 classes, 2 methods, 2 named. `SignalGetter` is
identical (`$$N` noise only). The daylight detector's
`updateSignalStrength` differs only in spelling: 26.1
calls `getEffectiveSkyBrightness`, 1.21.11 inlines
`getBrightness(SKY) - getSkyDarken()` — verified by
javap to be the same formula, so there is no
behavioural delta; the Rust already implements the
shared form. No code changed.

## Item 226 — world/inventory (rank 2, NO-CHANGE)

17 classes, 22 methods, 0 named. Sixteen anonymous
slot classes differ only by dropped null-check codegen
and `is` overload spellings (same tests); the
26.1-only `ContainerInput` class changes no slot
signature or body. No code changed.

## Item 227 — skins.rs (rank 2, NO-CHANGE)

10 classes, 11 methods, 1 named. The skin manager and
render cache differ only by visibility widening and
dropped null-check codegen; every method body
(including `registerTextures`) is bytecode-identical.
No code changed.

## Item 228 — mobs/phantom.rs (rank 2, NO-CHANGE)

8 classes, 11 methods, 1 named. `Phantom.canAttack`
and all seven inner goal classes are bytecode-identical
apart from visibility widening and dropped null-check
codegen. The two 1.21.11-only overrides are no-ops:
`getSoundVolume()` returns the 1.0f the 26.1 base
already returns (verified by javap), and
`canAttackType` returns true. No code changed.

## Item 229 — registry.rs / dimension_type.rs (rank 2, CHANGED)

1 class, 11 methods, 1 named.
`DimensionType.hasEnderDragonFight` and
`defaultClock` are 26.1-only codec fields (verified
absent from the 1.21.11 jar's dimension-type data), so
`DimensionFacts::from_json` now derives both from the
type id: the fight belongs to the End, and the clock
follows the adapter's 26.1-shaped mapping (Nether
none, End its own, Overworld the overworld's). The
`default_clock` field itself stays — it is the
adapter's internal clock model, not a wire field.
`getStorageFolder` already uses the 1.21.11
DIM-1/DIM1 layout; `CardinalLightType` keeps the same
values under its new home. New test pins the
derivation and the ignoring of 26.x-shaped keys.

## Item 230 — level/speleothem.rs (rank 2, CHANGED)

1 class, 11 methods, 1 named. `canGrow` in 1.21.11
wants a water BLOCK (source) two above the dripstone;
26.1 accepted any water fluid state, so a waterlogged
block two above grew dripstone there and not here.
`speleothem_grow` now tests the block plus
`is_source()` (the source check was also missing
against both versions). `maybeTransferFluid` and the
rest are identical. New level test: water-source
column grows, waterlogged-fence column does not (runs
in CI; skips locally without prepared resources).

## Item 231 — game/creaking.rs (rank 2, NO-CHANGE)

6 classes, 10 methods, 1 named. The creaking and its
heart block entity are identical apart from the Brain
provider restructure (item 218's pattern), the
PathType renames (item 200's pattern, same malus
values), `getRandom()` → the `random` field, and
dropped null-check codegen. The Rust's
`set_transient` already uses the 1.21.11 names. No
code changed.

## Item 232 — ticket_levels.rs (rank 2, CHANGED)

1 class, 10 methods, 1 named. The ticket logic is
identical (`ChunkPos.pack`/`unpack` spelling only).
The saved-data type id changes: 26.1's
`minecraft:chunk_tickets` (`data/minecraft/chunk_tickets.dat`)
is 1.21.11's plain `"chunks"` (`data/chunks.dat`,
flat). The save/load paths, the probe fixture and the
datafix type mapping (`"chunks"` → `SavedTickets`)
are ported; the forced-chunks test now pins the file
location. The other `saved_data_type` entries keep
their 26.x namespaced keys — their flat-layout port
belongs to their owning items.

## Item 233 — game/fangs.rs (rank 2, NO-CHANGE)

4 classes, 9 methods, 1 named. The fang renderer
differs only by the `CameraRenderState` package move
and a `submitModel` RenderType API difference (same
visual result); `considersEntityAsAlly` is the same
test under the `getType().is(...)` spelling; the
evoker diffs are visibility widening and dropped
null-check codegen. No code changed.

## Item 234 — game/sheep_dye.rs (rank 2, NO-CHANGE)

1 class, 9 methods, 1 named. The dye colour moves
from the stack's `DYE` component (26.1) to the item's
own `dyeColor` field (1.21.11); the interaction
itself is identical. The Rust already implements the
1.21.11 form in both paths (sheep dyeing from the item
name, sign dyeing from the item registry entry). No
code changed.

## Item 235 — game/trading.rs (rank 2, NO-CHANGE)

2 classes, 9 methods, 1 named. The offer-generation
mechanism differs (26.1's registry `TradeSet` + loot
context vs 1.21.11's hardcoded `VillagerTrades`
listings picked by random removal), but the mechanism
is inseparable from the tables, which are item 751's
work (rank 3); `merchant.rs` already documents that
`Trades` loads empty here. Everything else in
`AbstractVillager` is identical, and
`LookAndFollowTradingPlayerSink` is bytecode-identical.
No code changed.

## Item 238 — registries.rs (rank 2, NO-CHANGE)

4 classes, 8 methods, 1 named. 1.21.11 spells the
method `getCustomTabSugggestions` (three g's — a
Mojang typo), which the Rust already cites; the
26.1-only `ResourceArgument.getClock`/`getTimeline`
helpers have no Rust analogue (the parser's resource
argument is generic over the registry the command
tree names). No code changed.

## Item 239 — environment.rs (rank 2, NO-CHANGE)

2 classes, 8 methods, 1 named. The world-clock
removal (item 224's pattern): 1.21.11's
`addDefaultLayers` passes `level::getDayTime` as the
timeline sampler instead of a `ClockManager`; the
Rust already samples timelines at `day_time`
directly. No code changed.

## Item 240 — light.rs (rank 2, NO-CHANGE)

4 classes, 8 methods, 1 named. Pure renames:
`getLightDampening` -> `getLightBlock`,
`ChunkPos.x()`/`z()` -> fields, `pack` -> `asLong`,
plus visibility widening. The Rust already cites
`getLightBlock` and implements `isEdgeOccluded`.
No code changed.

## Item 242 — core/component (rank 2, NO-CHANGE)

10 classes, 18 methods, 0 named. The component
registry shrinks 110 -> 104 (six 26.1-only types:
`additional_trade_cost`, four sound variants,
`dye`); five types change shape (`EitherHolder` for
the variant/damage components, `TagKey` for banner
patterns, the `ProvidesTrimMaterial` record). The
Rust is value-generic over components, so the type
changes are invisible at its level; the forms it
reads are already the 1.21.11 ones, and component
ids come from the prepared 1.21.11 registries
report. The map/patch builder differences (validator,
prototype-resolving `get`) are Java plumbing with no
Rust analogue. No code changed.

## Item 243 — text_display_render_fixture.rs (rank 2, NO-CHANGE)

1 class, 7 methods, 1 named. The diff collapses to
two renames after stripping instruction numbers: the
background opacity read goes direct to `Options`
instead of through the extracted render state, and
`CameraRenderState` moves package. The Rust already
cites `Options.getBackgroundOpacity(0.25)`. No code
changed.

## Item 246 — signs.rs (rank 2, NO-CHANGE)

2 classes, 6 methods, 1 named. The sign model is
picked from the block identity instead of an
attachment point, the edit screen draws through
`GuiGraphics` instead of the extractor, and click
text resolves via `updateForEntity` instead of a
`ResolutionContext` — all the same result. The Rust
already picks the sign kind from the block name and
implements the click-command execution. No code
changed.

## Item 247 — terrain_cmds.rs (rank 2, NO-CHANGE)

2 classes, 6 methods, 1 named. `ChunkPos.pack` ->
`toLong`, and `LocationCheck` carries a condition
type instead of its own codec — a registration
pattern with no Rust analogue (the predicate is
keyed by its string type). `test` itself is
unchanged. No code changed.

## Item 248 — game/loom.rs (rank 2, CHANGED)

1 class, 6 methods, 1 named. Two real deltas under
the render-state extraction rename: the scrollbar
hover cursor checks only the 15-tall thumb in
1.21.11 (26.1 checked the whole 56-tall track and
gated on the scrollbar being active), and the dye
colour comes from `DyeItem.getDyeColor` instead of
the DYE component. Ported the thumb hover
(`LoomState::thumb_hovered` + a pointing-hand
request in `draw_loom`; vanilla's resize-while-
dragging has no Rust cursor variant yet) and updated
the doc citations to the 1.21.11 names. New test
`only_the_thumb_hover_asks_for_the_hand` (verified
by `cargo check --tests` plus a standalone run of
the verbatim logic — the full exe lib test binary
OOMs locally at -j 1).

## Item 249 — game/ominous_item_spawner.rs (rank 2, NO-CHANGE)

2 classes, 6 methods, 1 named. Pure noise: the
`CameraRenderState` package move and
`getRandom()` -> the `random` field for the 60-120
warmup roll, which the Rust already rolls from the
level random. No code changed.

## Item 250 — game/storage.rs (rank 2, NO-CHANGE)

2 classes, 6 methods, 1 named. The shulker screen
reaches the same 176x167 by `imageHeight += 1`
instead of constructor args, the background draws
through `renderBg`, and the sound pitch reads the
`random` field instead of `getRandom()` — all the
same result, all already the Rust's form. No code
changed.

## Item 251 — mobs/spawn_selection.rs (rank 2, NO-CHANGE)

4 classes, 6 methods, 1 named. Pure noise: `pack`
-> `toLong`, `ChunkPos.containing` -> the
constructor, plus visibility widening. The Rust
already implements `after_spawn` and the mob-cap
checks. No code changed.

## Item 253 — gfx/occlusion.rs (rank 2, NO-CHANGE)

1 class, 5 methods, 1 named. Pure renames:
`ChunkPos.x()`/`z()` -> fields, `pack` -> `asLong`,
plus visibility widening. No code changed.

## Item 254 — old_world_fixture.rs (rank 2, NO-CHANGE)

1 class, 5 methods, 1 named. Pure renames:
`ChunkPos.x()`/`z()` -> fields, `pack` -> `asLong`,
plus visibility widening. The blending logic is
identical. No code changed.

## Item 255 — trading_fixture.rs (rank 2, NO-CHANGE)

1 class, 5 methods, 1 named. The wandering
trader's `updateTrades` uses the hardcoded
`WANDERING_TRADER_TRADES` pairs in 1.21.11 instead
of the three trade sets — item 751's work with the
tables, as in item 235. The rest is spelling and
visibility noise. No code changed.

## Item 256 — game/inventory_slots.rs (rank 2, NO-CHANGE)

2 classes, 5 methods, 1 named. The container screen
reaches the same `114 + rows * 18` height by setting
`imageHeight` after construction, and crafting
`assemble` takes the registry lookup in 1.21.11 —
both already the Rust's form. No code changed.

## Item 257 — game/lan.rs (rank 2, NO-CHANGE)

2 classes, 5 methods, 1 named. Pure noise:
`ChunkPos.containing`/`unpack` -> constructors,
dropped null checks, visibility widening. No code
changed.

## Item 258 — game/nautili.rs (rank 2, CHANGED)

1 class, 5 methods, 1 named. The feed heal is twice
the fed item's FOOD nutrition (1.0 without one — a
fish bucket), in both versions: 26.1's `feed(..., 2,
1)` takes a multiplier and a default, which the Rust
had misread as a flat heal of 2.0. Ported via
`mobs/nautilus.rs::feed_heal` (cod heals 4, cooked
salmon 12, a bucket 1). The 26.1-only
`requiresCustomPersistence` override (tame persists)
is unobservable in 1.21.11 — taming always goes
through `interact`, which sets persistence required
first. New test
`feeding_heals_twice_the_nutrition_or_one_for_a_bucket`.

## Item 259 — mobs/creaking.rs (rank 2, NO-CHANGE)

1 class, 5 methods, 1 named. The brain provider
restructure (item 218's pattern) and the PathType
renames in `setTransient` (item 200's pattern) —
both already the Rust's form. No code changed.

## Item 260 — mobs/equip.rs (rank 2, NO-CHANGE)

1 class, 5 methods, 1 named. The brain provider
restructure (item 218's pattern) and the typed
`is(Item)` spelling in `wantsToPickUp` — the Rust
already implements the golden-axe-only pickup with
a test. No code changed.

## Item 261 — mobs/vex.rs (rank 2, NO-CHANGE)

5 classes, 5 methods, 1 named. The diff collapses
to dropped null-check codegen alone — the Vex class
and its four goal classes are otherwise
bytecode-identical. No code changed.

## Item 262 — client (rank 2, NO-CHANGE)

15 classes, 15 methods, 0 named. `FramerateLimiter`
is 26.1-only — the extracted form of the framerate
logic that 1.21.11 keeps inline in `Minecraft` and
the Rust already implements as `getFramerateLimit`.
The rest is null-check codegen, visibility widening
and the extractor rename. No code changed.

## Item 265 — beehive.rs (rank 2, NO-CHANGE)

2 classes, 4 methods, 1 named. `setBeeReleaseData`
is the same in both versions except 26.1 gates the
age update on the age lock — a 26.1-only system
(`AgeableMob` in 1.21.11 has no age-lock methods).
The Rust already implements the unconditional
1.21.11 form. No code changed.

## Item 266 — game/block_entity_anims.rs (rank 2, NO-CHANGE)

1 class, 4 methods, 1 named. Pure noise: the typed
`is(Block)` spelling and `getRandom()` -> the
`random` field in target selection and the animation
tick. No code changed.

## Item 267 — game/clone_block_ticks_tests.rs (rank 2, CHANGED)

5 classes, 4 methods, 1 named. The `/clone` area
check multiplies the spans as `int` in 1.21.11
(26.1 widens to `long`), so a huge region wraps —
4096-cubed wraps to 0 and passes. Ported via
`clone_area` (i32 `wrapping_mul`); the error now
reports the wrapped value too. New test
`the_clone_area_wraps_like_java_ints` (verified by
`cargo check --tests` plus a standalone run of the
verbatim logic).

## Item 268 — game/lan_entities.rs (rank 2, NO-CHANGE)

2 classes, 4 methods, 1 named. The display
transformation defaults to identity in both
versions (26.1's `Transformation.IDENTITY` field
vs 1.21.11's `identity()` static; record
accessors vs getters); the furnace minecart's
`interact` drops an unused hit-vector parameter
in 1.21.11. The Rust already implements the
1.21.11 form; one doc citation updated to the
1.21.11 spelling. `text_display`: 7 passed.

## Item 269 — game/mob_gear.rs (rank 2, NO-CHANGE)

2 classes, 4 methods, 1 named. `ItemStack.is`
overload tightening and `IntProvider` changing
from abstract class (1.21.11) to interface (26.1)
with the codec moving to `IntProviders` — pure
Java signature noise; the pillager's 1-in-300
piercing crossbow is already implemented. No
source change.

## Item 270 — mobs/breeze.rs (rank 2, NO-CHANGE)

1 class, 4 methods, 1 named. A single-line diff:
`BlockState.is(Object)` vs `BlockState.is(Block)`
in the breeze long-jump honey-block check — the
same overload tightening as item 269, no
behavioural delta. The honey refusal is already
implemented. No source change. `mobs::breeze`:
7 passed.

## Item 271 — spear.rs (rank 2, CHANGED)

3 classes, 4 methods, 1 named. One real line in
`UseParams.fromKineticWeapon`: 1.21.11 ends the
spear sway at the knockback window's start
(knockback + delay - 40), 26.1 at the dismount
finish — for the iron spear the sway ramps over
ticks 42..107, not 42..62. Ported; new test
`the_sway_runs_to_the_knockback_window`.
`multipart/Condition` only moved packages
(`model` -> `dispatch`); its codec is identical.
`spear`: 5 passed.

## Item 272 — worldgen/tree_blocks.rs (rank 2, CHANGED)

1 class, 4 methods, 1 named. `getOptionalDistanceAt`
anchors on `BlockTags.LOGS` in 1.21.11; 26.1
anchors on the `prevents_nearby_leaf_decay` tag
(which holds exactly the logs in 26.1, but does
not exist at all in 1.21.11 — so the Rust's tag
read resolved empty and logs counted as distance
7). Both leaf-distance reads now anchor on logs;
the dead `PREVENTS_LEAF_DECAY` flag and tag field
are removed. The existing
`crops_grow_and_leaves_decay` level test covers
the fix in CI. `worldgen::tree_blocks`: 2 passed.

## Item 273 — network/chat (rank 2, NO-CHANGE)

6 classes, 14 methods, 0 named. `ShowItem`
holds an `ItemStackTemplate` in 26.1 and an
`ItemStack` in 1.21.11 — the same
`{id, count?, components?}` shape, read
value-generically by the Rust; `Style`
visibility widening; the two missing-side
`$1` classes are 26.1-only compiler scaffolding
(a `ResolutionContext` switch map from item 98,
an anonymous signed-argument visitor whose outer
class is unchanged). No source change.
`component`: 13 passed; `text::`: 4 passed.

## Item 274 — clouds.rs (rank 2, NO-CHANGE)

1 class, 3 methods, 1 named. A large diff that
collapses to plumbing: 26.1 passes the cloud
range into `render` as a parameter (shifting
every local slot), 1.21.11 reads it from the
options; Blaze3D GPU types are classes in 26.1
and interfaces in 1.21.11; `prevType` renamed.
Zero differing constants — the mesh and render
sequence are identical. The Rust already passes
the range in through `Input`. No source change.
`clouds`: 12 passed.

## Item 276 — mipmap.rs (rank 2, NO-CHANGE)

1 class, 3 methods, 1 named. 26.1 passes the
transparency in as a `Transparency` record;
1.21.11 scans frame 0 itself (`hasTransparentPixel`,
any pixel with alpha 0). Same branch structure,
same constants — the Rust's caller-computed
`has_transparent` (same alpha-0 scan) is already
the 1.21.11 form in substance. No source change.
`mipmap`: 6 passed.

## Item 277 — random_source.rs (rank 2, NO-CHANGE)

1 class, 3 methods, 1 named. The unseeded
thread-local factory is `createThreadLocalInstance`
in 26.1 and `createNewThreadLocalInstance` in
1.21.11 (same `SingleThreadedRandomSource`
seeded from `ThreadLocalRandom`); the seeded
overload is 26.1-only (item 210). The Rust
already models the unseeded form; one doc
citation updated to the 1.21.11 name.
`random_source`: 1 passed.

## Item 278 — sound_events.rs (rank 2, NO-CHANGE)

3 classes, 3 methods, 1 named. The normalized
bytecode diff is empty: `Sound`,
`SoundEventRegistrationSerializer` and
`WeighedSoundEvents` are identical in both
versions. No source change. `sound_events`:
5 passed.

## Item 280 — game/native_block_entity_draw.rs (rank 2, NO-CHANGE)

2 classes, 3 methods, 1 named. Three renames:
`CameraRenderState` moves package, `blockState`
widens to public, and the light read is
`getLightCoords` in 26.1 vs `getLightColor` in
1.21.11 (same value, same default). One doc
citation updated to the 1.21.11 name.
`cargo check -p horizon-client` passes.

## Item 281 — game/quick_actions.rs (rank 2, NO-CHANGE)

3 classes, 3 methods, 1 named. `RegistryAccess`
itself (including `lookupOrThrow`) is identical;
the only diffs are dropped null-check codegen in
a constructor and a field widening. No source
change.

## Item 282 — menu/place.rs (rank 2, NO-CHANGE)

3 classes, 3 methods, 1 named. `typeHolder` vs
`getItemHolder` (one call, a rename) plus two
dropped null-check sequences; zero constant
differences — the `RecipePicker` lines are pure
renumbering. The Rust keys on the item id
directly. No source change. `menu::place`:
5 passed.

## Item 283 — level/portal.rs (rank 2, NO-CHANGE)

1 class, 3 methods, 1 named. Three occurrences
of the `BlockState.is(Object)` vs `is(Block)`
overload tightening (items 269/270) — the
portal-block count, `isEmpty`, and the obsidian
frame check. The Rust checks by block identity.
No source change. `portal`: 2 passed.

## Item 284 — worldgen/feature_region.rs (rank 2, NO-CHANGE)

1 class, 3 methods, 1 named. Three renames:
the `is` overload tightening, `ChunkPos.x()` /
`z()` methods vs fields, and `isInsideBuildHeight`
vs `isOutsideBuildHeight` (inverted predicate,
same branch). The Rust already has the 1.21.11
shapes. No source change.
`worldgen::generator_decoration`: 2 passed.

## Item 286 — block_entity_nbt.rs (rank 2, NO-CHANGE)

2 classes, 2 methods, 1 named. `fromStack`
takes a `HolderLookup.Provider` in 1.21.11
because `JukeboxPlayable.song` is an
`EitherHolder` (item 242's pattern) resolved
via `unwrap` — empty for an unregistered key;
26.1's is a pre-resolved `Holder`. The Rust
already resolves the song id through the
registry at every use and bails when it is
absent. `VaultConfig` statics lose `final` in
1.21.11 with no effect. No source change.
`block_entity_nbt`: 13 passed.

## Item 287 — nbt_path.rs (rank 2, NO-CHANGE)

1 class, 2 methods, 1 named. Pure visibility
widening on two error types and
`createTagPredicate`; `parse` is identical. No
source change. `nbt_path`: 12 passed.

## Item 288 — scoreboard.rs (rank 2, CHANGED)

3 classes, 2 methods, 1 named. `load` itself is
identical (dropped null check, field widening
only); the real delta is the saved-data type id
— 26.1's `ScoreboardSaveData.TYPE` is the
Identifier `minecraft:scoreboard`, 1.21.11's is
the plain string `"scoreboard"`, so the file is
`data/scoreboard.dat` flat, not
`data/minecraft/scoreboard.dat` (the item-232
pattern, left to this item there). The runtime
load/save path moved to flat, and
`saved_data_type` now maps the flat key too
(namespaced kept as fallback). New test
`saved_data_types_cover_both_layouts`.
`scoreboard`: 6 passed.

## Item 289 — overlay.rs (rank 2, NO-CHANGE)

1 class, 2 methods, 1 named. The normalized
bytecode diff is empty: `CreeperRenderer` is
identical in both versions. No source change.
`overlay`: 39 passed.

## Item 290 — game/lightning.rs (rank 2, NO-CHANGE)

1 class, 2 methods, 1 named. `Level.getRandom()`
vs the `Level.random` field — same source, same
draws; zero constant differences. The Rust
already reads the field directly. No source
change.

## Item 291 — game/sculk_fixture.rs (rank 2, NO-CHANGE)

1 class, 2 methods, 1 named. `serialize` is
rewritten between versions (26.1 streams values
through an anonymous consumer gated on
`isSingletonState()`; 1.21.11 iterates the
values map with `appendProperty` gated on an
empty property list) but produces the identical
string — verified piece by piece, including
that `Property$Value.valueName()` is
`property.getName(value)`. Plus the
`typeHolder`/`getBlockHolder` rename. No source
change. `block`: 2 passed.

## Item 292 — game_rules_screen.rs (rank 2, CHANGED)

12 classes, 12 methods, 0 named. 1.21.11 has
the single concrete EditGameRulesScreen,
opened only from CreateWorldScreen's More tab;
26.1 split it into AbstractGameRulesScreen +
WorldCreation/InWorld screens and added a
search box, the in-world variant (with the
set-game-rule packets, absent from the 1.21.11
jar) and a 36-pixel header. The Rust is
rewritten to the 1.21.11 form: no search, no
in-world variant, no confirm-on-close; Done
hands back the whole edited GameRules, Escape
and Cancel discard unconditionally; header 33
with the title centred; label split width 170
-> 175. Entries, tooltips, narration, sorting
and the box colours are identical in both
versions. `game_rules_screen`: 6 passed; the
title rules test: 1 passed.

## Item 293 — mobs/riders.rs (rank 2, NO-CHANGE)

1 class, 2 methods, 1 named. Two hunks of the
`is` spelling change: `updateControlFlags`
tests the rider against the raiders tag via
`LivingEntity.is` (26.1) vs `getType().is`
(1.21.11) — the same membership — and the
roar-target predicate excludes armor stands
via `is` vs `getType().equals`. The flag
assignments are identical. The Rust already
implements the raider rule species-wise.
`mobs::riders`: 13 passed.

## Item 294 — old_world.rs (rank 2, NO-CHANGE)

1 class, 2 methods, 1 named. The whole diff
is the recurring no-op pattern: ChunkPos
`x()`/`z()` methods vs fields, `pack` vs
`asLong`, and the `implements` clause order.
Zero constant differences. `old_world`: 1
passed.

## Item 295 — recipe_book.rs (rank 2, NO-CHANGE)

2 classes, 2 methods, 1 named. The
`typeSwitch` receiver is typed `RecipeDisplay`
in 26.1 and `Object` in 1.21.11 — the same
two-case switch — and the `TypeSettings`
record fields lose `private`. No source
change. `recipe_book`: 26 passed.

## Item 296 — version.rs (rank 2, NO-CHANGE)

1 class, 2 methods, 1 named. Protocol 775 ->
774, already the Rust's value; the debug-flag
set differs (1.21.11 adds
DUMP_INTERPOLATED_TEXTURE_FRAMES and
USE_WORKFLOWS_HOOKS, 26.1 adds CHAT_DISABLED
and PREFER_WAYLAND) but the Rust models no
SharedConstants debug flags; the rest is
static-init renumbering with identical
constants. No source change. `version`: 1
passed.

## Item 297 — collisions.rs (rank 2, NO-CHANGE)

1 class, 1 method, 1 named. A single-line
diff: the `typeSwitch` receiver in `of` is
typed `Entity` in 26.1 and `Object` in
1.21.11 — the same single-case switch. No
source change. `collisions`: 11 passed.

## Item 298 — command_block.rs (rank 2, NO-CHANGE)

1 class, 1 method, 1 named. Three
occurrences of the `BlockState.is(Object)`
vs `is(Block)` overload tightening; the mode
mapping is identical. No source change.
`command_block`: 4 passed.

## Item 299 — mesher/lighting.rs (rank 2, NO-CHANGE)

1 class, 1 method, 1 named. `ARGB.gray` is
26.1-only; 1.21.11's tinted-vertex path
multiplies the tint as a float and floors
once (`as8BitChannel`), differing by up to
one per channel. The Rust's `lit_channel`
already implements the 1.21.11 form and
documents the difference; one stale doc
citation updated to `as8BitChannel`.
`mesher::lighting`: 11 passed.

## Item 300 — falling_block_fixture.rs (rank 2, NO-CHANGE)

1 class, 1 method, 1 named. The moving-piston
check is the `BlockState.is` overload
tightening and the water check is
`FluidState.is(WATER)` vs `getType() ==
WATER` — the same equality. No source change.
`falling_block`: 7 passed, 1 ignored.

## Item 301 — game/narrator.rs (rank 2, NO-CHANGE)

1 class, 1 method, 1 named. `checkStatus`
shows the failed-TTS dialog via Blaze3D's
`MessageBox.errorWithContinue` in 26.1 and
via a direct `tinyfd_messageBox` call in
1.21.11 — the same message, the same
Continue semantics, the same exception on
declining. The Rust logs the message and
goes on (no native message box). No source
change. `cargo check -p horizon-client
--tests` passes.

## Item 302 — game/native_reload_listeners.rs (rank 2, NO-CHANGE)

1 class, 1 method, 1 named. Normalized
bytecode diff EMPTY: `prepare` is identical
in both versions. No source change. `cargo
check -p horizon-client --tests` passes.

## Item 303 — game/native_server.rs (rank 2, NO-CHANGE)

1 class, 1 method, 1 named. The only
difference is the order of the two
interfaces in the `implements` clause;
`withCallback`'s bytecode is identical. No
source change. `cargo check -p
horizon-client --tests` passes.

## Item 304 — game/spear.rs (rank 2, NO-CHANGE)

1 class, 1 method, 1 named. The large diff
collapses to one call: 26.1 reads the range
with `getAttackRangeWith(stack)`, 1.21.11
with `entityAttackRange()` — both are the
stack's ATTACK_RANGE or the entity default,
and the sole caller (`ItemStack.onUseTick`)
passes the active item either way. Zero
constant differences. No source change.
`spear`: 5 passed.

## Item 305 — game/villagers.rs (rank 2, NO-CHANGE)

1 class, 1 method, 1 named. A single line:
the `BlockState.is(Object)` vs `is(Block)`
overload tightening on the composter check.
No source change. `cargo check -p
horizon-client --tests` passes.

## Item 306 — mobs/living.rs (rank 2, NO-CHANGE)

1 class, 1 method, 1 named. Normalized
bytecode diff EMPTY: `Vec3i.compareTo` is
identical in both versions. No source
change. `position`: 7 passed, 1 ignored.

## Item 307 — mobs/spawner.rs (rank 2, CHANGED)

1 class, 1 method, 1 named. The spawner's
sky-light check uses the raw sky light in
1.21.11; 26.1 subtracts the sky darken
(`getEffectiveSkyBrightness`), which
relaxes upper-bounded limits by up to 11
at night. `valid_position` now reads the
raw sky light. `mobs::spawner`: 7 passed.

## Item 308 — projectiles.rs (rank 2, NO-CHANGE)

1 class, 1 method, 1 named. A single line:
the `BlockState.is(Object)` vs `is(Block)`
overload tightening on the bubble-column
check. No source change. `projectiles`: 44
passed; `mod_thrown_types_are_found_by_id_and_item_once_registered`
fails identically on the clean tree
(pre-existing, not chased).

## Item 309 — structure_start_nbt.rs (rank 2, NO-CHANGE)

1 class, 1 method, 1 named. Two lines:
ChunkPos `x()`/`z()` methods vs fields;
the tag keys are identical. No source
change. `worldgen::structure::start`: 14
passed.

## Item 310 — worldgen/blending.rs (rank 2, NO-CHANGE)

1 class, 1 method, 1 named. Three lines:
`CELL_COLUMN_COUNT` loses `private`, and
two `BlockState.is` calls on the mushroom
blocks are the overload tightening. No
source change. `worldgen::blending`: 3
passed.

## Item 312 — world/entity (rank 2, NO-CHANGE)

11 classes, 11 methods, 0 named. `Avatar`
gains `getProfile()` and goes abstract in
26.1; `EntityFluidInteraction` is a 26.1-only
extraction of the fluid logic 1.21.11 keeps
inline on `Entity`; `CAN_BE_PICKED` drops
its spectator exclusion in 26.1 — none of
the three has a Rust surface (the Rust's
pick iterates only non-player mobs, which
can never be spectators; one comment
updated to say so). The rest is visibility
widening and codegen noise. `mobs::living`:
21 passed, 1 ignored.

## Item 318 — client/resources/model/cuboid (rank 2, NO-CHANGE)

10 classes, 10 methods, 0 named. The 26.1
cuboid package is a package move + rename
of 1.21.11's `renderer/block/model/BlockElement*`
classes: paired bytecode is identical after
renaming (Euler rotations included — they
exist in 1.21.11 too). `MissingCuboidModel`
is `MissingBlockModel` with the same
construction. The Rust's `CuboidModel` /
`ElementRotation` already match the 1.21.11
element system. `horizon-assets model`:
29 passed.

## Item 330 — resources/RegistryDataLoader (rank 2, NO-CHANGE)

6 classes, 8 methods, 0 named. The
26.1 task-based loader (`LoaderFactory`,
`ResourceManagerRegistryLoadTask`) is
absent in 1.21.11, which loads directly
via `loadContentsFromManager`; the
sorted-by-Identifier registration order
still holds (the `listMatchingResources`
map is sorted). Two citations re-pointed
to the 1.21.11 loader. `crafting`: 39
passed, 1 ignored; `lan::data`: 7 passed,
1 ignored.

## Item 346 — client/gui/screens/worldselection (rank 2, NO-CHANGE)

5 classes, 6 methods, 0 named.
`GameTab`/`WorldTab` identical apart from
name noise; `WorldOpenFlows$1Data` is
visibility only. `FileFixerProgressScreen`
is 26.1-only (1.21.11 has no file-fixing
step — the upgrade UI is the chunk-based
`OptimizeWorldScreen`), but the Rust
already disables the flow for 1.21.11:
`set_file_fixing` has no callers, so the
retained screen never shows. `create_world`:
6 passed; `level_loading`: 12 passed.

## Item 349 — com/mojang/blaze3d/vertex (rank 2, NO-CHANGE)

4 classes, 6 methods, 0 named. `BufferBuilder`'s
fast-format check and normal gate, the
`mulPose(Transformation)` overload, and the
normal-matrix fast-path check
(`checkPropertyRaw(16)` vs `isOrthonormal`
— JOML property 16 is `PROPERTY_ORTHONORMAL`)
are internal optimisations with identical
results; `QuadInstance` is 26.1-only. The
Rust has no `PoseStack`/`BufferBuilder`
surface. `item_render`: 47 passed, 2 ignored.

## Item 350 — world/item/crafting/display/SlotDisplay (rank 2, CHANGED)

4 classes, 6 methods, 0 named. The codec
used the 26.3 slot-display numbering (11
types); rewired to 1.21.11's 8 (item 2,
item_stack 3, tag 4, smithing_trim 5,
with_remainder 6, composite 7), dropping
`WithAnyPotion`, `OnlyWithComponent` and
`Dyed`. The 1.21.x adapter tables lose
their hand-corrected runs (now identity);
the 26.x tables gain the 11->8 mapping,
so the three 26.x-only displays map to
nothing. The resolver arms and the LAN
fixture registry follow. `packets::recipes`:
1 passed; `adapter::ids`: 11 passed;
`lan::advancements`: 13 passed;
`recipe_book`: 26 passed; `stations`:
26 passed.

## Item 355 — client/renderer/block/dispatch (rank 2, NO-CHANGE)

5 classes, 0 methods named. The 26.1
dispatch package is the restructure of
1.21.11's `BlockStateModelLoader` +
multipart system, which the Rust already
implements (`horizon-assets`' blockstate
module: variants, weighted sets,
conditions). No dispatcher surface in
the tree. `horizon-assets`: 57 passed.

## Item 356 — client/renderer/blockentity/state (rank 2, NO-CHANGE)

5 classes, 5 methods, 0 named. 26.1
carries the chest orientation as a
`Direction` and the sign wood/transforms
in the render state; 1.21.11 carries a
float angle and derives the sign data at
render time — which is what the Rust
already does (`y_rot` from the facing,
sign transforms from the block state).
`block_entity_render`: 34 passed, 1 ignored.

## Item 357 — com/mojang/blaze3d/audio (rank 2, NO-CHANGE)

2 classes, 5 methods, 0 named. 26.1
adds `SoundBuffer.size`/`format()`/
`isValid()` and the `DeviceList` record;
the shared buffer methods are unchanged,
and the Rust has its own `DeviceProbe`
with the same shape. `audio`: 30 passed.

## Item 359 — world/item/enchantment (rank 2, NO-CHANGE)

5 classes, 5 methods, 0 named. The empty
exclusive set is `HolderSet.empty()` in
26.1 and `direct(new Holder[0])` in
1.21.11 — the same empty set; the rest is
visibility widening and two 26.1-only
internal effect-action interfaces.
`enchant`: 17 passed.

## Item 371 — client/gui/GuiGraphicsExtractor (rank 2, NO-CHANGE)

5 classes, 4 methods, 0 named. 26.1's
`GuiGraphicsExtractor` is 1.21.11's
`GuiGraphics` renamed, methods included
(`itemDecorations`→`renderItemDecorations`,
`centeredText`→`drawCenteredString`,
`map`→`submitMapRenderState`, and so on);
decompiled bodies are identical. All 28
Rust citations re-pointed to the 1.21.11
names. `hud_overlays`: 6 passed.

## Item 372 — util/worldupdate/UpgradeProgress (rank 2, NO-CHANGE)

4 classes, 0 methods named. The 26.x
file-fixer progress handle has no 1.21.11
counterpart; the Rust's merged handle
keeps its file-fixer half inert (the
screen never shows) while its counters
serve the live 1.21.11 `WorldUpgrader`
port. Two stale doc comments fixed.
`datafix`: 32 passed, 2 failed (the known
pre-existing pair).

## Item 377 — IME (TextInputManager / PreeditEvent) (rank 2, NO-CHANGE)

3 classes, 0 methods named. The 26.1 IME
preedit system has no 1.21.11 counterpart;
the Rust already implements the 1.21.11
form — committed text only, the OS shows
the composition window (`game/ime.rs`).
`ime`: 73 passed, 3 ignored.

## Item 388 — client/renderer/block/FluidModel (rank 2, NO-CHANGE)

3 classes, 3 methods, 0 named. 26.1's
`FluidModel` replaces `LiquidBlockRenderer`,
which 1.21.11 keeps (five hardcoded water/
lava sprites, no per-fluid model). The
Rust's renderer already follows the
`LiquidBlockRenderer` form; the two
`FluidModel` citations on the mod-fluid
declaration were reworded to the liquid
renderer's sprites. `fluid`: 1 passed.

## Item 389 — world/clock + waypoints/ChunkWaypoint (rank 2, NO-CHANGE)

3 classes, 3 methods, 0 named. The
ChunkWaypoint diff is the ChunkPos
accessor pattern; the world-clock system
is 26.1-only, and the SetTime packet
difference (1.21.11's day time +
tick flag vs 26.1's per-clock states) is
already translated both ways by the
adapter. `translate::v1_21_11`: 1 passed;
`step_1_21_11`: 5 passed.

## Item 390 — valueproviders/IntProviders + CappedProcessor (rank 2, NO-CHANGE)

2 classes, 3 methods, 0 named. The
`IntProvider` interface-vs-class
difference and the codec's move to
`IntProviders` are 26.1-only; the
CappedProcessor's random is LegacyRandom
in 1.21.11 (Xoroshiro in 26.1), which
the Rust already uses. Two citations
re-pointed. `processor_bounds`: 2 passed.

## Item 392 — client/resources/model/sprite (rank 2, NO-CHANGE)

3 classes, 0 methods named. The 26.1
sprite-material system (`Material$Baked`,
`MaterialBaker`, `SpriteId`) has no
1.21.11 counterpart (1.21.11 resolves
`Material` through `MaterialSet`) and no
Rust surface — the Rust's `Material` is
its own texture reference. `resolve`:
12 passed.

## Item 393 — commands/arguments/blocks (rank 2, NO-CHANGE)

2 classes, 3 methods, 0 named. The
`BlockInput` line is the `is` overload
tightening; `BlockStateParser$1` is
26.1's serialize consumer, already
covered by item 291 (identical outputs).
`block`: 2 passed.

## Item 404 — model/cuboid/CuboidModel (rank 2, NO-CHANGE)

2 classes, 0 methods named. `CuboidModel`
is the pure rename of 1.21.11's
`BlockModel` (bytecode identical after
collapsing packages); the Rust's
`CuboidModel` already matches (item 318).
`model`: 29 passed.

## Item 406 — world/entity/Entity inner classes (rank 2, NO-CHANGE)

3 classes, 2 methods, 0 named.
`EntityPathElement`/`Movement` differ
only in visibility; `Flags` is a 26.1-only
empty annotation interface. `mobs`: 706
passed, 43 ignored.

## Item 420 — renderer/block quad output (rank 2, NO-CHANGE)

2 classes, 0 methods named. Both
26.1-only (`BlockQuadOutput`,
`FluidStateModelSet`); the Rust's mesher
cites 1.21.11's `ModelBlockRenderer`
directly and has no surface for either.
`mesher`: 71 passed, 24 ignored.

## Item 426 — world/entity/ai/memory (rank 2, NO-CHANGE)

2 classes, 2 methods, 0 named. 26.1's
`MemorySlot` replaces 1.21.11's
`ExpirableValue`, but `canExpire`,
`hasExpired` and the tick timing are
identical (a ttl-N memory is visible for
exactly N ticks in both); the added
`nearbyEntities()` accessor is a plain
getter. All 11 `MemorySlot` citations
re-pointed to `ExpirableValue`. `allay`:
10 passed, 4 ignored.

## Item 439 — world/clock/WorldClocks (rank 2, NO-CHANGE)

1 class, 0 methods named. `WorldClocks`
is 26.1-only. The End sky flash ticks on
game time in 1.21.11 (the End's clock in
26.1) — the Rust already uses game time
with LegacyRandom per cycle. The
`end_clock` doc wrongly called it the
flash's driver; corrected. Noted: on a
1.21.11 server `day_time` never updates
in the Nether/End (the packet carries
only clock 0) — narrow, needs a
clock-architecture decision, not this
item. `world::`: 3 passed.

## Item 442 — renderer/block/BlockStateModelSet (rank 2, NO-CHANGE)

1 class, 0 methods named. 26.1-only; the
particle sprite comes from
`BlockStateModel.particleIcon()` in
1.21.11 (no baked-model classes exist in
the 1.21.11 jar). The one citation
re-pointed. `block_particles`: no tests
under that filter; lib build clean.

## Item 457 — templatesystem/loader/TemplateSource (rank 2, NO-CHANGE)

1 class, 0 methods named. 26.1-only; the
template reading lives on
`StructureTemplateManager` in 1.21.11,
with the same `unlimitedHeap` +
`readCompressed` form. The one citation
re-pointed. `nbt::`: 14 passed.

## Item 462 — gui/components/events/GuiEventListener (rank 2, NO-CHANGE)

1 class, 1 method, 0 named. 26.1's added
`preeditUpdated` default is part of the
26.x-only IME preedit system (item 377);
the Rust already takes only committed
text. `ime`: 73 passed, 3 ignored.

## Item 466 — renderer/state/GameRenderState (rank 2, NO-CHANGE)

1 class, 0 methods named. 26.1-only
(the render-state container); 1.21.11
renders immediate-mode and the Rust has
no surface for it. `hud`: 149 passed, 1
failed — the HDR screenshot pixel test
(float drift on llvmpipe), pre-existing
and environmental, not this item (no code
touched).

## Item 483 — dimension/end/EnderDragonFight (rank 2, CHANGED)

1 class, 7 methods named. 1.21.11's
`EndDragonFight` is a plain class whose
state (`EndDragonFight$Data`) lives in
level.dat's `DragonFight`, not a
SavedData file: the fight now loads
from `Data.DragonFight` with the
1.21.11 PascalCase keys (`Gateways` a
list, `Dragon` for the UUID, only
`IsRespawning` for the respawn — no
stage or time) and saves by handing the
built tag to a new `Level.dragon_fight`
field, written with the level write
(the `scheduled_events` pattern); the
26.x `.dat` file is a read fallback
only. `tryRespawn` searches 2 blocks
out, not 3; the respawn stage ticks
whenever players are present (only the
scan, dragon search and crystal count
wait for the arena); a respawn loaded
from the save re-finds its crystals on
the first arena-loaded tick instead of
aborting when none is alive. All 24
`EnderDragonFight` citations re-pointed
to `EndDragonFight`. `fight_data`: 4
passed.

## Item 487 — templatesystem/loader/ResourceManagerTemplateSource (rank 2, NO-CHANGE)

1 class, 0 methods named. 26.1-only
(the whole `templatesystem/loader/`
package — `TemplateSource`,
`DirectoryTemplateSource`,
`ResourceManagerTemplateSource` — is
absent from both 1.21.11 jars);
template reading lives on
`StructureTemplateManager` in 1.21.11
(item 457), which is what the Rust
already cites. No citations anywhere.
`nbt::`: 14 passed.

## Item 503 — client-render/fog.rs (rank 3, CHANGED)

5 classes, 70 methods, 5 named.
`FogRenderer.setupFog`/`computeFogColor`
are a restructure only (26.1 writes
the colour into `FogData.color` and
returns the data; 1.21.11 returns the
colour and inlines the buffer upload)
with identical bodies, and
`AtmosphericFogEnvironment` differs
only in where the panoramic forward
vector comes from. The one port:
`LightmapRenderStateExtractor` is
26.1-only — its torch flicker lives in
`LightTexture` in 1.21.11 as
`blockLightRedFlicker` (same formula,
same `RandomSource.create()`), so
`FogState.block_light_flicker` is
renamed `block_light_red_flicker` and
all 6 citations re-pointed to
`LightTexture` (fog.rs, game.rs,
sculk.rs, end.rs); none remain.
`fog::`: 16 passed; `cargo check -p
horizon-client --tests` clean.

## Item 504 — mobs/spear_use.rs (rank 3, NO-CHANGE)

5 classes, 11 methods, 9 named.
The diff is visibility only (`private
final` → package-private) except
`SpearAttack`'s constructor: 1.21.11
takes a 4th float, an approach
distance (PiglinAi passes 10.0F), that
no method ever reads — dead state 26.1
dropped. The one edit updates the
`PiglinSpear` doc to the 1.21.11 4-arg
construction. `spear`: 5 passed.

## Item 505 — command/codec.rs (rank 3, NO-CHANGE)

11 classes, 25 methods, 5 named.
`ResourceOrIdArgument` (all 5
`parse`), `HolderSetCodec` and
`RegistryFileCodec` are byte-identical.
The loot classes differ only in
registration plumbing (`LootPoolEntryType`
constants vs `bootstrap`, `MAP_CODEC` →
`CODEC`, `codec()` → `getType()`) and
in `validate` (validation-problem
output only; the Rust decodes, it does
not validate loot). 1.21.11's
`InvertedLootItemCondition
.getReferencedContextParams` has no Rust
surface (no context-param tracking
anywhere). `codec`: 3 passed.

## Item 506 — item_render/gui.rs (rank 3, NO-CHANGE)

8 classes, 31 methods, 3 named.
The PiP renderers differ only in
package moves (`renderer/state/gui` →
`gui/render/state`), the sprite →
material system, and the projection
buffer restructure (same ±1000 ortho);
`Lighting` differs only in the
GpuDevice class/interface form and the
`CardinalLighting$Type` →
`DimensionType$CardinalLightType` move
(same values). `GuiItemAtlas` is
26.1-only, but 1.21.11's `GuiRenderer`
holds the items atlas itself and
`GuiItemRenderState` carries the same
scissor/oversized bounds (verified
identical) — the one edit re-points
the module-doc citation there.
`item_render::gui`: 8 passed.

## Item 507 — datafix/world_upgrader.rs (rank 3, NO-CHANGE)

6 classes, 30 methods, 3 named.
26.1 extracted the upgrader into
`RegionStorageUpgrader` (+ `Builder`,
`TagModifier`), `FileToUpgrade` and the
`UpgradeProgress` family; 1.21.11 keeps
it in `WorldUpgrader`'s inner
`AbstractUpgrader` / `ChunkUpgrader` /
`SimpleRegionStorageUpgrader` classes
with inline progress fields, and
`verifyChunkPos` is inline in the chunk
upgrader. The per-chunk behaviour is
identical (order, counts, status keys,
erase-cache drops, POI default 1945,
recreate replacement), and vanilla's
single-threaded round-robin vs the
Rust's per-dimension workers cannot
change the bytes (files are
independent). Doc citations re-pointed
to the 1.21.11 homes; the 26.1-shaped
internal structure stays.
`world_upgrader`: 13 passed.

## Item 508 — assets/blockstate/choice.rs (rank 3, NO-CHANGE)

8 classes, 55 methods, 0 named.
26.1 moved the whole multipart system
to `renderer/block/dispatch/` (item
355); paired by hand, the only deltas
are the 26.1-only sprite system
(`particleMaterial` vs 1.21.11's
`particleIcon`) and `materialFlags`,
plus visibility. The selection logic
(weighted pick, multipart re-seed,
first-selector particle) is identical,
and the Rust already cites
`particleIcon`. horizon-assets: 57
passed.

## Item 509 — game/debug_lines.rs (rank 3, CHANGED)

11 classes, 34 methods, 2 named.
`SectionRenderDispatcher.getStats` is
`"pC: %03d, pU: %02d, aB: %02d"` in
1.21.11 — the middle field is the
pending-upload queue, dropped by 26.1's
uber-buffer restructure. The F3 chunk
line now carries `pU` from
`pending_uploads.len()`. The
`LevelExtractor.*Statistics` citations
re-pointed to
`LevelRenderer.getSectionStatistics` /
`getEntityStatistics`. The chunk-generation
cache is the Rust's own optimization
(1.21.11 recomputes; same lines).
`cargo check -p horizon-client
--tests` clean.

## Item 510 — game/dispenser.rs (rank 3, CHANGED)

17 classes, 34 methods, 2 named.
All the anonymous behaviours are
identical (compared by decompile —
the `$N` numbering shifts between
versions). The one delta is spawn
eggs: 26.1 routes them through the
default method (an egg without
ENTITY_DATA drops as an item);
1.21.11 registers one anonymous
behaviour for all eggs, and an egg
without ENTITY_DATA stays in the
dispenser (`getType` returns null).
Eggs moved into the `dispense_item`
match as a registered arm, and
`dispense_spawn_egg` returns the stack
unchanged when ENTITY_DATA was
removed. `cargo check -p
horizon-client --tests` clean.

## Item 511 — hud/pack_text.rs (rank 3, NO-CHANGE)

5 classes, 41 methods, 1 named.
`GlobalSettingsUniform.update` is
identical except 1.21.11 takes a
`Camera` where 26.1 takes its `Vec3`
(same values, same Std140 writes).
`GuiRenderer`'s big diff is the
atlas/PiP restructure (item 506); the
GUI ortho projection (1000/11000,
flip-Y, ORTHOGRAPHIC) and the -11000
model view are the same in both.
`pack_text`: 9 passed.

## Item 512 — game/commands/text_commands.rs (rank 3, NO-CHANGE)

6 classes, 30 methods, 2 named.
`NbtContents` differs structurally
(26.1: record + CompilableString path
+ `plain` flag for rich SNBT styling;
1.21.11: class + compiled path +
`asString` — string raw, else compact
SNBT), but the Rust already implements
the 1.21.11 form with a test.
`ScoreContents` is CompilableString vs
SelectorPattern with identical logic;
tellraw/title differ only in the
resolution API. `text_commands`: 4
passed.
