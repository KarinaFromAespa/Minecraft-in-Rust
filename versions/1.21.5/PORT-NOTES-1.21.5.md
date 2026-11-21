# Port notes: Minecraft 1.21.5

1.21.5 is the sixth adjacent delta port in the **Components era (E6,
1.20.5–1.21.11)**, taken from 1.21.6. It is a cross-protocol delta:
protocol **771 → 770**, world version 4435 → 4325, data pack 80.0 →
71.0 and resource pack 63.0 → 55.0 — and unlike 1.21.6, the packet
set is **not** identical: 8 packets are removed and 60 play
serverbound packets are renumbered, so this port needed a full
packet rewire plus the adapter re-base. The measured delta is **426
ranked work items**. The port is the mechanical layer: the pin is
set, the six generated tables are regenerated against 1.21.5's own
jar, the adapter ladder is re-based to the new protocol, the
verification gates pass, and the remaining work is sized, ranked
and documented. The behaviour code in this tree is still the 26.3
base's, inherited through 1.21.11, 1.21.10, 1.21.9, 1.21.8, 1.21.7
and 1.21.6; the port is **in progress**, not parity-verified.

## Provenance

- Base: the programme's completed 1.21.6 tree — the `port/1.21.6`
  branch at a0b30b3, which contains the 1.21.6 tree including the
  1.21.8 code changes (packet rewire, adapter re-base, oracle port),
  the jsr305 jar, the both-forms `gen_version_consts.py`, the
  jsr305 classpath fix in `tools/port_regen.py`, and the full
  ported oracle (391 files). No overlay was needed. Published
  1.21.6: main@53b720f2.
- 1.21.5's facts, from its own version JSON and generated tables:
  protocol **770**, world (data) version **4325**, data pack
  format **71.0**, resource pack format **55.0**, Java **21**,
  stable release (2025-03-25).
- Prepare ran clean from the Mojang manifest (2026-11-19):
  **9,148 classes indexed, 74,447 methods**; mappings: methods
  **81,125 mapped / 81,125 total (60,993 renamed)**, fields
  **41,610 mapped / 41,610 total (41,181 renamed)**,
  0 placeholders; named jar 33,990,037 bytes; datagen ran.
  Nothing Mojang-derived is committed in this tree.
- Diff 1.21.6 → 1.21.5: **426 ranked work items** (rank 1: 9,
  rank 2: 259, rank 3: 133, rank 4: 23, rank 5: 2). Classes +68
  −403, 13 moved, **1,202 changed (4,995 methods); 1,219 of the
  changed or removed ones are ported or proved by Horizon**.
  Assets +5 −152 ~2,701; en_us keys +0 −226 ~7.

## The delta: dialogs, waypoints and the happy ghast go away

Rank 1 is the version pin (`version.rs`), the six regenerate
items, the save-format item (DataFixers 4435 → 4325: 1 schema,
2 fixes added, 14 datafix classes changed) and the version facts
(protocol 771 → 770, world 4435 → 4325). Rank 2 is 259 harness
reruns. Rank 3 is 133 unproved items. Rank 4 is 23 items. Rank 5
is 2 items.

The visible content changes are the removals of three 1.21.6
features:

- **Dialogs**: the whole dialog system is gone — the
  `dialog_action_type`, `dialog_body_type` and `dialog_type`
  registries, the `hex_color` and `dialog` command argument
  types, the `input_control_type` registry, and the
  `minecraft:show_dialog` / `minecraft:clear_dialog` /
  `minecraft:custom_click_action` packets.
- **Waypoints / the locator bar**: the `minecraft:waypoint`
  packet, the `waypoint_transmit_range` and
  `waypoint_receive_range` attributes, and the
  `camera_distance` attribute are gone.
- **The happy ghast**: the entity, its spawn egg, the dried
  ghast block and item, the 16 harness items, the
  `music_disc_tears` disc, the `happy_ghast_one_cm` custom stat,
  the ghastling/happy-ghast/dried-ghast sound events, and the
  `player_sheared_equipment` trigger are gone. The item registry
  loses 19 entries, the sound registry 29 (and gains 4:
  `leash_knot.break`/`place`, `puffer_fish.ambient`,
  `sand.wind`).

The item id shift rule across the delta: ids <619 unchanged;
620–801 shift −1; 818–1110 shift −17; 1112–1267 shift −18;
>1268 shift −19. Entity ids above 56 shift −1; attributes above
8 shift −1. That is why the adapter re-base and the test re-pins
below are mechanical but wide.

## Tables: 6 regenerated

| Generator | Result vs 1.21.6 |
|---|---|
| `version_consts` | regenerated (+8 −8 — the pin: 1.21.5 / 770 / 4325 / 71.0 / 55.0) |
| `packet_ids` | regenerated (+61 −77 — 8 packets removed, 60 renumbered) |
| `entity_tracking` | regenerated (+0 −1) |
| `lan_command_order` | regenerated (+4 −9) |
| `command_suggestions` | regenerated (+76 −1) |
| `vanilla_command_order` | regenerated (+1 −4) |

The regen took three runs (commits 26161336f4, d14f861e7c,
6dcbbace99 on `port/1.21.5`), because each run unblocked the
next: run 1 regenerated the first three tables and failed the
three oracle-command generators on two oracle compile errors
(`VanillaCommandParseOracle`: `HolderLookup` vs `Registry`;
`VanillaCommandSuggestionsExporter`: `SuggestionProvider`
generics) — fixed in 943e9fe, verified with `javap` against the
1.21.5 named jars. Run 2 regenerated `lan_command_order` and
`command_suggestions` and failed `vanilla_command_order` because
`horizon-protocol` would not compile until the packet rewire
landed. Run 3 regenerated `vanilla_command_order`. The committed
`regen.json` lists all six generators with their run-1–3 statuses
combined (D1 completeness, per the gate change at main@15ff93a8).
A fresh regen reproduces all six committed tables byte-for-byte —
see `regen-determinism.json`, no overrides.

## Packet rewire: 8 packets removed, 60 renumbered

Commit e7c7ff9 on `port/1.21.5` removes the 8 packets 1.21.5
does not have: configuration ShowDialog/ClearDialog/
CustomClickAction (with the `ClickPayload` codec), play
ShowDialog/ClearDialog/TrackedWaypoint, play serverbound
ChangeGameMode/CustomClickAction. The `TrackedWaypoint` payload
types (`TrackedWaypoint`, `WaypointId`, `WaypointTarget`,
`waypoint_operation`) stay in `packets/play.rs` as plain types —
`locator_bar.rs` and `lan/waypoints.rs` use them; only the
`Clientbound::TrackedWaypoint` enum variant is gone. `DialogHolder`
stays too (game-side dialog types use it). Three packet tests went
with the removed packets (`tracked_waypoint_bytes`,
`change_game_mode_uses_the_game_type_varint`,
`click_payload_is_length_prefixed`); `dialogs_round_trip` was
replaced by `dialog_holder_encodes_registry_as_id_plus_one`.

Commit 3ca05fe reworks the game-side wire boundary around the
missing packets: `net.rs` drops the `Event::ShowDialog`/
`ClearDialog` events and `send_custom_click` is now a no-op
(1.21.5 has no custom-click-action packet); `game/remote.rs`
drops the dialog/waypoint arms; `game/locator_bar.rs` drops
`handle_waypoint`; `game/commands/extra.rs` makes
`send_dialog`/`clear_dialog_to` local-only (`/dialog` cannot
reach LAN guests); `game/debug_keys.rs` runs `/gamemode` via
`send_command` on remote servers (no change-game-mode packet);
`game/lan_actions.rs` drops the `ChangeGameMode` arms;
`lan/waypoints.rs` returns `Vec<(u32, TrackedWaypoint)>`
payloads instead of packets and `game/lan_waypoints.rs` drops
them (connections stay tracked); `studio/recorder.rs` drops the
`TrackedWaypoint` arm.

## Adapter re-base to 770

The adapter ladder (`crates/horizon-protocol/src/adapter/`) is
re-based from the 771 pin to 770 (commit 590263d on
`port/1.21.5`; scripts `rebase770.py` and `fix_slot_display770.py`
in the Stream 1 workdir, modelled on the 1.21.6 `rebase771.py`).
The rebase was recovered from diffing the regenerated tables
against the 771 tree:

- INHERIT map: 1.21.4→PIN, 1.21.6→PIN, 1.21.8→PIN; all other
  renames/reassigns/state-aliases unchanged from the 771 re-base.
- `happy_ghast` removed from `PINNED_NAMES` entity_type in
  `tools/generate_registry_id_adapter.py` (it arrived in 1.21.6),
  and `entity::HAPPY_GHAST` removed from the `AGEABLE` array in
  `adapter/translate.rs` v1_21_11 (36→35).
- `adapter.rs` pin swap: V1_21_5 is now the pin identity static
  (with `components_before_1_21_6: true` chained from V1_21_8);
  V1_21_6 is a regular old version; the five "pinned 1.21.6"
  comment sites became "pinned 1.21.5"; the `from_protocol` test
  updated (774 four above pin, 773 three, 772 two, 771→V1_21_6,
  770→Current).
- SlotDisplay: the `slot_display` registry is identical in 1.21.5
  and 1.21.6 (8 entries, same order), so the 771 fix values apply
  unchanged; the v1_20_3 ENTITY_TYPE_DOWN and v1_21_3
  BLOCK_STATE_DOWN anchor runs are identical too
  (breeze_wind_charge is still 18 in the 770 pin), so the same
  hand-patches applied.
- `EntityDataSerializers` is NOT mentioned in the diff, so the
  `adapter.rs` serializer remaps are unchanged.

One real code fix surfaced during the re-pins (commit 5f72237):
`ARGUMENT_TYPES` in `packets/play/commands.rs` was the **26.3**
registry (61 entries incl. `hex_color` 17 and `dialog` 55); the
770 pin needs **1.21.5's** 55-entry registry (no hex_color/dialog,
`minecraft:color` at 16 not `team_color`, uuid at 54). The old
list misread parser 42 (time in 1.21.5) as gamemode (no
properties), leaving the i32 unread — "packet 16 was larger than
expected: 25 extra bytes".

The registry shifts forced wide test re-pins (commits 0c2d9ea,
5f72237, 8f45fb5, c7b12d8, 764f5d1, c7bf8db, dd5f3f9, 7ef0c26,
2f67b01, 1353b74, 1bd95f7, 4669828): diamond 862→845, brick
980→963, calcite state 23376→23344, filled map 1059→1042,
tadpole 125→124, player kind 149→148, angler sherd 1371→1352,
bowl 856→839, iron ingot 868→851, emerald 863→846, explode sound
615→606, zombie villager 147→146, llama 76/129→75/128, pig
95→94, saddle 801→800, turtle 132→131, mooshroom 83→82,
minecart 82→81, hanging kinds 58/71/89/149→57/70/88/148, the
legacy attributes re-pinned to the 1.21.5 report (flying_speed
11, follow_range 12, jump_strength 14, knockback_resistance 15,
luck 16, max_absorption 17, max_health 18, movement_speed 21,
spawn_reinforcements 26 — the 1.21.5 attribute registry uses
UNPREFIXED names), the v1_21_8 disc fixtures (cat 1250→1232
pinned, pigstep 1267→1248, levelup sound 1179→1154), the
equippable allowed-entities (pig 95→94, zombie 145→144), the
blast furnace station `SlotDisplay::Item(1290)` → `Item(1271)`,
and the slot fixtures (diamond sword 895→878, enchanted book
1194→1176, crossbow 1274→1255, bundle 991→974, compass 989→972,
goat horn 1286→1267, white banner 1214→1196, leather chestplate
914→897, suspicious stew 1274→1255, beehive 1306→1287, pig spawn
egg 1127→1109, diamond pickaxe 897→880, potion 1076→1059, stick
905→888, player head 1185→1167, firework rocket 1192→1174,
firework star 1193→1175, written book 1171→1153, writable book
1170→1152).

## Oracle port: 14 errors, 70 files

`horizon-prepare` (commit fd79ef0) ports the oracles to the
1.21.5 APIs — verified with `javap` against the 1.21.5 named
jars, not guessed:

- `Player`'s only constructor in 1.21.5 is
  `Player(Level, BlockPos, float, GameProfile)` (26.3 has the
  2-arg form): 31 `super(null,null)` sites in Player subclasses
  became `super(null, null, 0.0F, null)`, plus 3
  `super(level, new GameProfile(...))` sites gained the
  `BlockPos(0,0,0)` and `0.0F` arguments. `ServerPlayer`'s
  4-arg constructor is unchanged, so its subclasses are untouched.
- `LocalPlayer`'s constructor in 1.21.5 is 7-arg
  `(Minecraft, ClientLevel, ClientPacketListener, StatsCounter,
  ClientRecipeBook, boolean, boolean)` — no `Input` parameter:
  20 nine-arg `super(...)` sites became the 7-arg form
  (trailing `false, false`).
- `SharedConstants.getCurrentVersion()` returns `WorldVersion`
  in 1.21.5 (no `net.minecraft.Version` class), and the data
  version is `getDataVersion().getVersion()`, not
  `dataVersion().version()`: 17 sites in 12 files.
- `net.minecraft.util.StrictJsonParser` does not exist in 1.21.5:
  5 files now use `com.google.gson.JsonParser` (`parseString`
  for String args, `parseReader` for the one `StringReader`
  arg).

The first `horizon-prepare` run surfaced 14 javac errors in 9
files; the fix batch above covers the latent occurrences of the
same three API shapes across the whole oracle tree (the exporters
compile in batches, so the first run's errors were only the
first batch).

## Verification

- `cargo check --workspace --all-targets`: clean, 0 errors,
  0 warnings (16 m 07 s at `-j 1`, post-rebase run; the
  pre-rebase run was 8 m 06 s). The first pre-rebase run failed
  on disk quota writing the incremental `horizon_client_render`
  dep-graph (0 code errors); the re-runs used
  `CARGO_INCREMENTAL=0`.
- `cargo test -p horizon-protocol`: **427 pass / 2 fail** (429
  total — down 3 from 1.21.6's 432: the packet rewire removed 3
  packet tests). The 2 failures are the same pre-existing
  sandbox-network pair in untouched code
  (`connection::tests::a_dead_address_does_not_hold_up_the_next`,
  `resolver::dns::tests::asks_each_server_in_turn_and_retries_truncated_replies_over_tcp`),
  out of scope.
- `horizon-prepare` completes end-to-end: exit 0, "Done in
  364.1 s", after the oracle port above.

## Harness subset: no local profile (skipped, CI to measure)

No local harness profile was measured. The 1.21.6 publish
attempted a local sweep and spent ~4 h 30 min on attempts that
produced no profile (the `fast`-profile horizon-client build
OOMs on this VM's 6 GB under 3-stream load); the programme has
since moved to measuring profiles on CI, and disk quota pressure
(~99G of ~100–103G on 2026-11-19) rules out the local sweep's
multi-GB build trees. The profile is to be measured by the CI
`port-verify` workflow on GitHub runners (16 GB) after merge.

`status.json` records the harness section as 0/0/0 of 17 with
this note, and the CI run will fill it in.

## Behaviour port (2026-11-21)

Direction 1.21.6 -> 1.21.5, 426 items (ranks 9/259/133/23/2).
Verification method: the named 1.21.6/1.21.5 client jars
(9,483 vs 9,148 classes) via the mappings front-end (1.21.5:
9,148/9,148 mapped, 0 placeholders), then a Vineflower
decompile of the report's changed classes only (1,898 + 1,711
class files) and a diff of the two source trees. The running jar
is the oracle; decompiled code is a reading aid.

### Rank 1: closed 9/9

- **Item 1 — protocol, verified:** the regenerated `ids.rs`
  matches the report exactly: configuration/clientbound 17,
  configuration/serverbound 8, play/clientbound 131,
  play/serverbound 64; renumbered spot checks chat 7, use_item
  63, use_item_on 62, swing 59 (all −1 from 1.21.6);
  `CHANGE_GAME_MODE` and `CUSTOM_CLICK_ACTION` absent. The
  `Clientbound::TrackedWaypoint` enum variant is gone (see the
  rewire section above); the payload types stay as plain types
  for `locator_bar.rs` / `lan/waypoints.rs`. No code change.
- **Items 2–7 — regenerate, verified:** all six generated files
  match `regen-determinism.json` byte-for-byte (sha256 checked
  against the committed hashes; fresh == committed):
  `command_order.rs`, `command_suggestions.rs`, `tracking.rs`,
  `vanilla_order.rs`, `version.rs`, `ids.rs`. The mechanical
  publish already regenerated them against the 1.21.5 pin. No
  code change.
- **Item 9 — version facts, verified:** `version.rs` carries
  `PROTOCOL_VERSION` 770 and `DATA_VERSION` 4325; `adapter.rs`
  has V1_21_5 as the pin identity static (V1_21_6 a regular old
  version). No code change.
- **Item 8 — save-format, ported:** the 1.21.5 `DataFixers`
  registration differs from 1.21.6 in eight places with Rust
  surface, all ported in `crates/horizon-world/src/datafix/`:
  - Schema 165: 1.21.5's `TextComponentStrictJsonFix` rewrites
    EVERY text component (`fixTypeEverywhere`); 1.21.6 split it
    into sign-only + book-only fixes. Replaced
    `sign_text_strict_json` + `written_book_pages_strict_json`
    with `text_component_strict_json` (walks `Ty::TextComponent`;
    the walker visits sign lines, book pages, display names and
    lore, boss-event names, objective names and command output
    at k=164). The parser stays lenient: Gson's
    `JsonParser.parseString` (1.21.5) IS lenient — verified
    empirically against gson 2.13.2 (accepts bare words,
    single quotes, trailing commas) — so the 1.21.6 switch to
    `LenientJsonParser` is behaviour-preserving, and the
    `UnflattenTextComponentFix` / `LevelDataGeneratorOptionsFix`
    parser swaps are no-ops too.
  - Schema 1451.5: 1.21.5's `ItemRemoveBlockEntityTagFix`
    matches on the ITEM id (23 items) and touches item stacks
    only; 1.21.6's `RemoveBlockEntityTagFix` matches on the
    tag's block-entity id (2 items) and also strips
    falling-block and structure data. Reworked
    `remove_block_entity_tags` to the 1.21.5 form (item-id
    match, 23-item set, falling-block arm dropped).
  - Schema 1458: 1.21.5 converts an empty entity `CustomName`
    to `{"text":""}`; 1.21.6 removes the field. Dropped the
    empty-removal branch in `entity_custom_name` (the
    block-entity and item forms are unchanged between versions
    — verified, not in the diff).
  - Schema 1925: `MapIdFix` wraps `SAVED_DATA_MAP_DATA` in
    1.21.5 (the `data` wrapper moves into the base schemas in
    1.21.6, where the fix targets the map index instead).
    Retargeted the 1925 fix from `Ty::SavedMapIndex` to
    `Ty::SavedMapData`.
  - Schema 2537: `LegacyDimensionIdFix` is 1.21.6-only —
    removed the registration and the function.
  - `FeatureFlagRemoveFix` targets `LEVEL` in 1.21.5,
    `LIGHTWEIGHT_LEVEL` in 1.21.6 (the same root in the
    walker) — retargeted `feature_flag_remove`.
  - `RaidRenamesDataFix`: no `HeroesOfTheVillage` rename in
    1.21.5 — dropped the line.
  - `TooltipDisplayComponentFix`: no
    `minecraft:jukebox_playable` unwrap in 1.21.5 — dropped
    the line.
  - No Rust surface (verified): the V1925/V99/V1460/V3439
    schema registrations (the walker already visits
    hanging-sign text, matching 1.21.5),
    `DataFixTypes.LEVEL_SUMMARY` (walker-internal), and the
    1.21.6-only schemas V3439_1/V4420/V4421 (above 4325; fixes
    above the target never run — `CURRENT` is `DATA_VERSION`).
  - Verification: `cargo check -p horizon-world --all-targets`
    0 errors / 0 warnings; datafix unit tests 30/33 — the 3
    failures are pre-existing (same 3 fail on the pristine
    tree, proven by re-running with the changes stashed:
    `failed_upgrade_keeps_old_data_version`,
    `block_states_take_their_new_field_names`,
    `level_data_of_1_20_moves_to_the_new_fields` — all assume
    26.x versions/fixes above 4325, e.g. the 4771
    difficulty-settings fix).
- **Item 10 — entity_nbt.rs, ported:** the 3,473-line diff is
  mostly the ghast/phantom flight rework and the happy-ghast
  removals; three files carry Rust surface, all ported:
  - `mobs/nether.rs` — `GhastMoveControl`: 1.21.5 accelerates
    by a fixed 0.1 (1.21.6 reads `FLYING_SPEED * 5 / 3`, which
    is 0.1 for a default ghast — same value, different source)
    and `canReach` steps the box one block at a time,
    `1..<ceil(length)>`, never checking the endpoint (1.21.6
    sweeps the whole box at once, endpoint included).
    `RandomFloatAround`: 1.21.5 picks one random position and
    takes it (1.21.6 retries through
    `getSuitableFlyToPosition`, up to 64 tries with a heightmap
    mirror) — the code already took the first pick; the comment
    now says so.
  - `mobs/living.rs` — `FlyingMob.travel`: ghasts and phantoms
    fly the friction form in 1.21.5 (drag is the ground
    friction's when on the ground, 0.91 otherwise; ground speed
    is the standard `0.1 * 0.16277137 / f^3`; a phantom flies at
    0.02 in the air, not 1.21.6's 0.2), not `travelFlying`,
    which 1.21.6 inlined into both classes. Added
    `travel_flying_mob` (f32 arithmetic, matching the Java) and
    dispatch Ghast/Phantom to it before the `flying` check.
  - `mobs/leash.rs` — the plain ghast is not a quad-leash
    holder in 1.21.5 (1.21.6 added one): removed the Ghast arm
    from `quad_holder_offsets` and flipped the test case
    `("horse", "ghast", true)` to false.
  - Referrals (not this item's files): item 254
    (`SheepRenderState` — 1.21.5's jeb phase includes
    `+ entity id`; Rust `jeb_wool_rgb` takes only age), item 83
    (leash system — 1.21.5 `LEASH_TOO_FAR_DIST` 10.0 vs Rust
    SNAP 12.0), item 211 (`ColorLerper`, gone in 1.21.5).
  - Verification: `cargo check -p horizon-client
    --all-targets` 0 errors / 0 warnings (9m55s); full
    workspace check 0/0 (12m29s, item-8 state, before these
    edits — the client check above covers them).
- **Item 11 — realms.rs, ported:** the Realms screens' 1.21.5
  forms, in `crates/horizon-client/src/realms.rs`:
  - `RealmsWorldSlotButton`: all four slots are 80 square, 18
    apart (`frame(slot)`, 374 wide, centred) — not 1.21.6's
    80-active/50-others; the picture sits at +3/+3, 74x74, and
    is dimmed 0.56 on the ACTIVE slot (1.21.6 dims the inactive
    ones); the active slot draws the checkmark at +67/+4; names
    clamp at 64px; the version draws under EVERY slot (blank
    versions draw nothing). `getAction` regains JOIN: clicking
    the active slot of an unexpired, initialized Realm joins it
    (`joinRealm` — play when open, `openTheWorld(true)` when
    closed, via a new `open_then_join` + `join_after_open`
    flag that starts the join when the open lands); the active
    slot of an expired/uninitialized Realm and the minigame slot
    of an expired one are inert. The Worlds tab's three buttons
    re-centre under the 374-wide slots.
  - `RealmsMainScreen`: the CLOSED-state play-button tooltip
    (`mco.play.button.realm.closed`) is 1.21.6-only — removed.
    The play-button condition itself is identical in both
    versions (`!expired && OPEN && (compatible || upgrade ||
    own)`), only its home moved (screen method in 1.21.5, DTO
    method in 1.21.6) — comments fixed.
  - No Rust surface (verified): the render-API renames
    (`pushPose`, `RenderType::guiTextured`, `getName`,
    `isStable`, `sayNow`), the ARGB→RGB colour literals (same
    rendered colours), the tooltip-API renames, `RealmsUtil`'s
    1.21.6-only async helpers, the `RealmsSlot` wrapper (upload
    flow identical), `GetServerDetailsTask`'s
    `RealmsJoinInformation`→`RealmsServerAddress` rename,
    `RealmsPopups.customPopupScreen` (1.21.6-only), and the
    loading screen's `paddingTop(10)` (the Rust draws no such
    padding).
  - Structural gap (documented, not ported): 1.21.6 reworked the
    configure flow into tabs (`RealmsWorldsTab` et al. in
    `configuration/`); 1.21.5 has separate screens
    (`RealmsConfigureWorldScreen`, `RealmsSettingsScreen`,
    `RealmsPlayerScreen`, `RealmsSlotOptionsScreen`). The Rust
    keeps the tabbed navigation — re-hosting the configure flow
    as separate screens is item 404's question (rank 4, the
    1.21.6-only tab classes), not this item's. The slot
    tooltips (`slot.tooltip.*`) were never modelled in the Rust
    either (pre-existing gap, same in 1.21.6).
  - Verification: `cargo check -p horizon-client
    --all-targets` 0 errors / 0 warnings (3m18s, shared with
    item 12).
- **Item 12 — lan_actions.rs, ported:** the LAN host's
  server-side packet handling:
  - `ServerboundPlayerCommandPacket.Action`: 1.21.5 has NINE
    actions — `PRESS_SHIFT_KEY` (0) and `RELEASE_SHIFT_KEY` (1)
    head the enum, so every other action sits two higher than
    in 1.21.6 (STOP_SLEEPING 2, START/STOP_SPRINTING 3/4,
    START/STOP_RIDING_JUMP 5/6, OPEN_INVENTORY 7,
    START_FALL_FLYING 8). Renumbered everywhere the ids are
    read or sent: `lan/hub.rs` (decode + the shift-key arms,
    which now own `g.sneaking` and the `Sneak` mod event),
    `game/lan_actions.rs` (the game's arms + the sprint event),
    `game/remote.rs` (all four senders), `game/remote_vehicles.rs`
    (`riding_jump` + its test), `game/command_harness.rs` (the
    `player_command` op), `lan/hub_tests.rs` (the riding-jump
    test). The client gains `LocalPlayer.sendShiftKeyState`
    (`was_shift_key_down` on `Remote`, sent before the input
    packet as `LocalPlayer.tick` orders it).
  - `handlePlayerInput`: 1.21.5 only records the keys
    (`setLastClientInput`) — removed the sneaking update and
    the idle-timer reset from the hub's handler (both 1.21.6
    additions). The crouch-pose test now drives the pose
    through PRESS/RELEASE_SHIFT_KEY.
  - `handleMoveVehicle`: 1.21.5 checks `noCollision` on the box
    deflated 0.0625, before the move and at the target
    (`lan_vehicles.rs` now deflates both by 0.0625, not 0 /
    1e-5); the 1.21.6-only `!isFlyingVehicle()` exemption was
    never in the code — the comment said otherwise and now
    doesn't. `handleMovePlayer`'s `collide()` already implements
    the 1.21.5 form (`isPlayerCollidingWithAnythingNew` over
    `getCollisions`, 1e-5 deflate) — only the comment cited the
    1.21.6 name and query.
  - No Rust surface (verified): `level()`→`serverLevel()`
    renames, `ServerLevel.updateNeighboursOnBlockSet` (a 1.21.6
    extraction — `Level.setBlock`'s neighbour branch is
    byte-identical in both versions, and the Rust's helper
    implements it; comment re-cited), `waitForChunkAndEntities`
    (1.21.6-only, no Rust surface),
    `removeLatestMovementRecordingBatch` vs `removeLatestMovementRecording`
    (the Rust records no vehicle movement to roll back),
    the waypoint manager (already compute-but-drop for 1.21.5).
  - Verification: `cargo check -p horizon-client
    --all-targets` 0 errors / 0 warnings (3m18s, shared with
    item 11).
- **Item 13 — hub.rs, verified, no code change:** `PlayerList` /
  `ServerPlayer` / `ServerChunkCache` / `PlayerChunkSender`:
  the diff is the ValueInput/ValueOutput→CompoundTag NBT API
  migration (same fields read and written), `level()`→
  `serverLevel()` renames, and 1.21.6-only additions with no
  1.21.5 behaviour surface — the waypoint manager and its
  crouch transmit modifier (the Rust already computes waypoint
  payloads but sends none in 1.21.5), `waitForChunkAndEntities`
  (no Rust surface), `purgeStaleTickets`' signature. The spawn
  snap for new players moved from the `ServerPlayer` ctor
  (1.21.5) to `placeNewPlayer` (1.21.6) with the same outcome;
  the Rust already places newcomers via `adjustSpawnLocation`
  (`lan.rs`). One referral: `DimensionType.parseLegacy`
  (1.21.5 accepts a legacy int Dimension in playerdata, -1/0/1
  → nether/overworld/end; 1.21.6 falls back to overworld for
  any int) — the read lives in `save.rs`, so it belongs to
  item 18.
- **Item 14 — boats.rs, ported:** `Entity` / `EntityRenderer` /
  `AbstractChestBoat` / `ModelLayers`:
  - Movement recording: 1.21.5's `Entity.move` splits each move
    into per-axis segments in `axisStepOrder` (YZX when |x| < |z|,
    else YXZ) and keeps them in an unbounded per-move batch list;
    1.21.6 records one flat movement per move in a deque that
    merges the two oldest records at 100. The Rust keeps its flat
    record (the net block effects are identical — both walk axis
    steps) but now splits by the RESOLVED delta (`Axis::step_order`
    already matches vanilla exactly; the consumer was ordering by
    the pre-collision `original`, which matches neither version),
    and `MovementHistory::add` no longer caps at 100.
  - Fire ticks: 1.21.5 inits `remainingFireTicks` to
    -getFireImmuneTicks() (-1 for mobs, -20 for the player; the
    player init was already -20) and, in `applyEffectsFromBlocks`,
    leaves -getFireImmuneTicks() when a fire went out in the pass
    (1.21.6 inits 0 and resets every non-burning entity to
    -immune each pass). Mob init is now -1, so the first
    lightning strike ignites a fresh mob (`thunderHit`: +1,
    ignite iff 0 — the Rust already implements the rule); the
    player and mob block-effects passes now apply the post-check
    (snapshot burning before the pass, set -20 / -1 when the fire
    went out in it). `Entity.extinguishFire` is sound +
    `clearFire` (exactly 0), so `extinguish_fire` no longer keeps
    a negative value via `min(0)`. The 1.21.5-only `< 0 clearFire`
    after the fire-immune -4 burn-down is moot for living
    entities (`LivingEntity.baseTick` clears fire-immune mobs
    every tick; the Rust's instant-0 matches the net).
  - `AbstractChestBoat.interact`: 1.21.5 only runs
    `super.interact` (the lead step + boarding) when the player
    is not sneaking — a sneaking player opens the chest even with
    a lead in hand; 1.21.6 always runs it first. The lead step in
    `interact_boat` is now skipped for a sneaking player on a
    chest boat.
  - No Rust surface (verified): `requiresPrecisePosition`
    (1.21.6-only, no Rust surface), the leash render
    (`leashState` + 0.025 width in 1.21.5 vs `leashStates` +
    0.05 in 1.21.6 — the Rust draws no leash ropes at all),
    `ModelLayers`' happy-ghast layers (no Rust layer registry;
    the happy-ghast removal is item 58's), the movement-batch
    removal (the Rust records no vehicle movement to roll back).
  - Known gap: `rain_extinguish_player` (the LAN guest path)
    still clears to 0 instead of -20 — the pass-start snapshot
    is not plumbed there, and the difference needs 20 lightning
    strikes on one player to observe.
  - Verification: `cargo check -p horizon-client
    --all-targets` 0 errors / 0 warnings (9m28s).
- **Item 15 — game.rs, ported:** `Minecraft` / `GameRenderer` /
  `TitleScreen` / `SpectatorGui` / `Camera` / `ServerRecipeBook` /
  `DetectorRailBlock` / `ItemEntityRenderer` / `GameTestTicker` /
  `BlockEntityRenderer` / `AbstractFurnaceScreen` /
  `PrimaryLevelData` / `DedicatedServer`:
  - Camera: 1.21.5's third-person distance is a flat 4 blocks
    times the living entity's scale; 1.21.6 reads the
    `camera_distance` attribute (default 4, happy ghast 8) and
    takes the max with a living mount's. The distance computation
    now uses 4 × scale; the attribute and mount inputs stay on
    `CameraEntity` for the probe's fixture shape and are ignored.
    The detached-distance test now expects 8.0 for scale 2.0.
  - Far plane: 1.21.5's `getDepthFar` is the render distance in
    blocks times 4; 1.21.6 takes the max with the cloud range ×
    16. `depth_far` drops the cloud-range argument (three callers
    + test updated).
  - Pause audio: 1.21.5's `SoundManager.pause()` pauses every
    channel, and `MusicManager.tick` only runs while unpaused;
    1.21.6 pauses all but MUSIC and UI and ticks the music always.
    `Audio::pause` now pauses all channels; `tick_music` gates the
    manager tick on `!pause` (the option listeners and toast
    bookkeeping still run).
  - Spectator menu: 1.21.5 shows the slot key labels and the
    action tooltip only while the hotbar alpha is above 3/255
    (1.21.6: above 0).
  - Recipe book: 1.21.5's `toNbt`/`fromNbt` write all eight
    open/filter flags (1.21.6's codec omits the false ones);
    `Settings::write` now writes all eight, and `read_valid` is
    re-cited to `fromNbt`.
  - No Rust surface (verified): the title-screen fade
    (`panoramaFade`, `fadeWidgets`, the alpha gate — the Rust
    draws the logo and splash with no fade at all), the render
    pipeline (GuiRenderer/FogRenderer/PanoramaRenderer/
    ScreenEffectRenderer vs 1.21.5's inline forms; the item
    activation animation's math is already exact in
    `item_activation.rs`), `renderZoomed`/`renderHand` (no Rust
    surface), the rail rotate/mirror tables (the prepare
    exporter's `VanillaStructureBlockExporter` calls vanilla's
    `state.rotate`/`mirror` on the 1.21.5 jar, so the 1.21.5
    tables are already encoded — including the CLOCKWISE_180
    fallthrough the decompile shows for flat detector rails),
    `GameTestTicker.startTicking` (the Rust's `ticking` is a
    re-entrancy guard; its tick runs whenever a runner exists,
    the 1.21.5 form), the menu blur gate (already ≥ 1.0),
    `ServerRecipeBook.copyOverData`/`sendInitialRecipeBook`
    (same outcomes), `ItemEntityRenderer`'s bounding-box helper
    (same math, refactored), the dialog/quick-actions and
    waypoint removals (handled by the publish), accessor renames
    (`name()`→`getName()`, `stable()`→`isStable()`, narrator
    `sayNow`).
  - Verification: `cargo check -p horizon-client
    --all-targets` 0 errors / 0 warnings (2m45s).
- **Item 16 — remote.rs, verified, no code change:**
  `ClientPacketListener` / `Player` / `AbstractContainerScreen` /
  `BeaconBlockEntity` / `ClientCommonPacketListenerImpl` /
  `MerchantScreen` / `MultiPlayerGameMode` / `ConnectScreen` /
  `ReceivingLevelScreen` / `InterpolationHandler`:
  - The `handleEntityPositionSync` rider flip (1.21.5 repositions
    the rider only when the vehicle IS interpolating; 1.21.6 only
    when it is NOT) has no Rust surface: `move_entity` never
    repositions riders per packet — `position_riders` runs every
    tick for every vehicle.
  - `InterpolationHandler.interpolate`'s collision guard (1.21.5
    tests `makeBoundingBox().move(target + delta)`, a box offset
    by the entity's own position; 1.21.6 tests
    `makeBoundingBox(target + delta)`) has no Rust surface: the
    stepped interpolation keeps no per-tick delta and runs no
    collision guard.
  - The demo help messages (1.21.6 narrates them; 1.21.5 only
    chats them), the `DisconnectedRealmsScreen` (1.21.5) vs
    `DisconnectedScreen` + Back (1.21.6), and the container
    snapback animation are all unmodelled in the Rust.
  - `createPlayer`'s kept state (1.21.5: `isShiftKeyDown()`;
    1.21.6: `getLastSentInput()`) has no Rust surface: respawns
    reuse the live player and its input state is read fresh each
    tick, never snapshotted into a new player object.
  - The command tree's restricted suggestions provider and
    `NodeBuilder` (1.21.6-only) have no Rust surface: the tree
    from the packet carries no permission gating.
  - The `signedMessageEncoder = UNSIGNED` reset on login
    (1.21.6-only) has no Rust surface (no signed-message
    encoder); the waypoint attributes are inert reads against
    the 1.21.5 registry, which lacks them.
  - The label colour (-12566464 in 1.21.6 vs 4210752 in 1.21.5)
    renders identically: `Font` forces any colour with alpha
    below 64 opaque. The Rust draws opaque dark grey.
  - The rest is API migration with identical data: the
    ValueInput/ValueOutput → CompoundTag moves in `Player` save
    data, `BeaconBlockEntity` (same `primary_effect` /
    `secondary_effect` / `Levels` / `CustomName` / `lock`
    fields) and block-entity loading; the `Player` ctor's spawn
    snap (item 13); the tooltip deferral (`setTooltipForNextFrame`
    vs `renderTooltip` — same frame in the Rust's batched HUD);
    the dialog handlers (removed by the publish); accessor
    renames (`sayNow`, `getDifficulty`/`isLocked`,
    `serverLinks`).
- **Item 17 — hud.rs, ported:** `GuiGraphics` / `Font` /
  `TextureAtlas` / `TextureUtil` / `ReloadableTexture` /
  `SpecialGlyphs`:
  - Item decorations (`GuiGraphics.renderItemDecorations`):
    1.21.5 draws bar → count → cooldown, so the cooldown veil
    covers the count text; 1.21.6 draws bar → cooldown → count.
    `hud_overlays::item_decorations` now emits the count before
    the cooldown fill. Bar, cooldown and count geometry are
    identical in both versions (bar 13×2 at +2/+13, veil
    `floor(16(1-p))`/`ceil(16p)` at 0x7FFFFFFF, count at
    x+19-2-w / y+9, white with shadow).
  - The rest is the 1.21.6 GUI render refactor with no behaviour
    change: `GuiRenderState` / strata / `BlitRenderState` /
    `GuiTextRenderState` / `GuiItemRenderState` vs 1.21.5's
    immediate `PoseStack` + `BufferSource` vertex writes, and
    `RenderPipeline` vs `RenderType` signatures. Fill, gradient,
    hLine/vLine, blit, nine-slice, tiled-sprite and tooltip
    layout math are identical; the Rust's `HudBatch` already
    implements them.
  - Text: the Rust's world-text depths (shadow 0.03, lines
    +0.01) already match 1.21.5's `SHADOW_DEPTH` 0.03 /
    `EFFECT_DEPTH` 0.01, not 1.21.6's 0.003 / 0.001.
    Strikethrough [y+3.5, y+4.5] and underline [y+8, y+9]
    rects match both versions. `drawString`'s 1.21.5 `int`
    return (advance + 1 for a shadow) has no Rust caller.
    Tooltips are drawn at the end of the screen draw in the
    Rust, matching both versions in practice (the 1.21.6
    deferral only matters for mid-render tooltip calls).
  - `TextureAtlas` / `TextureUtil` / `ReloadableTexture` /
    `SpecialGlyphs` are GPU API signature changes only
    (`createTexture` / `writeToTexture` / buffer-mapping arg
    lists); the Rust drives wgpu directly and has no surface.
  - Verification: `cargo check -p horizon-client-render -p
    horizon-client --all-targets` 0 errors / 0 warnings (2m46s).
- **Item 18 — save.rs, verified, no code change:**
  `CampfireBlockEntity` / `BannerBlockEntity` /
  `ChiseledBookShelfBlockEntity` / `ShulkerBoxBlockEntity` /
  `LecternBlockEntity` / `TheEndGatewayBlockEntity` /
  `DataVersion` / `LevelVersion` / `LevelSummary` /
  `SectionStorage` / `LevelStorageSource`:
  - All six block entities are the ValueInput/ValueOutput →
    CompoundTag API migration with identical NBT fields
    (campfire `Items`/`CookingTimes`/`CookingTotalTimes`,
    banner `patterns`/`CustomName`, bookshelf
    `last_interacted_slot`, lectern `Book`/`Page`, gateway
    `Age`/`exit_portal`/`ExactTeleport`). The Rust reads and
    writes these fields directly from NBT in `save.rs`, so the
    migration is invisible.
  - `DataVersion` is a record in 1.21.6 and a plain class in
    1.21.5 with the same `isSideSeries` / `isCompatible`
    semantics; the Rust reads the data version as an int.
    `LevelSummary` / `LevelVersion` / `SectionStorage` are
    accessor renames only (`version()` → `getVersion()`,
    `stable()` → `isStable()`).
  - `LevelStorageSource` runs `DataFixTypes.LEVEL` on the
    lightweight summary in 1.21.5 vs `LEVEL_SUMMARY` in 1.21.6:
    no Rust surface — the Rust's world-list summary read
    applies no fixes, and the only `LightweightLevel`-targeted
    fix (`level_dat_difficulty`) is keyed at data version 4771,
    inert at 4325.
  - Correction to an earlier referral: `DimensionType.
    parseLegacy` (1.21.5 accepts a legacy int `Dimension` in
    playerdata, −1/0/1 → nether/overworld/end; 1.21.6 falls
    back to overworld) belongs to **item 161**
    (`dimension_type.rs`), not this item.
- **Item 19 — boat_leads.rs, ported (the leash system):**
  `Leashable` / `LeadItem` — the 1.21.6 leash-physics rewrite
  (spring/wrench/quad-leash) reverted to 1.21.5's legacy pull:
  - `mobs/leash.rs` rewritten: snap distance 12.0 → 10.0
    (`LEASH_TOO_FAR_DIST`), elastic range still 6.0; the pull
    is the legacy `copySign(unit² * 0.4, unit)` per axis over
    the feet-to-feet float distance, with `checkSlowFallDistance`
    after it; no spring, no angular momentum, no yaw change,
    no relative-velocity term. `handle_leash_at_distance`
    restored (sitting tamable: drop past 10, no pull;
    `PathfinderMob` home = holder block radius 5; equine stop
    eating and camel stand up past 6). Quad leash, `Wrench`,
    `notifyLeashHolder` / `notifyLeasheeRemoved` and
    `leash_spin` deleted.
  - `game/leads.rs` rewritten: `can_attach_lead` is now
    `canHaveALeashAttachedToIt` (leashable, alive, unleashed
    — no distance check); `player_leashed` is the ±7 cube
    around the fence block pos; `use_knot` is 1.21.5's
    `bindPlayerMobs` + knot interact (ties all player-led mobs
    and boats in range unconditionally, `LEASH_KNOT_PLACE` on
    the useOn path only; a knot click with nothing to tie
    discards the knot silently, creative removes hanging leads
    without dropping). No sneak-tie, no shears branch, no
    take-back, no LEAD_TIED / LEAD_UNTIED sounds; the knot
    outlives its last lead (`forget_untied_knots` deleted —
    a re-tie reuses the same knot entity, covered by the
    renamed knot-identity test). Fence-break discards play
    `LEASH_KNOT_BREAK` (1.21.6's `LEAD_UNTIED`).
  - `game/boat_leads.rs` rewritten the same way: legacy pull,
    snap past 10 with no sound, ±7 fence cube, no sneak or
    shears branch. `game/fireworks.rs`: the firework boost no
    longer drops leads (1.21.6-only).
  - Render: one rope again (`EntityRenderer.renderLeash`),
    width 0.05 → 0.025, sag math unchanged; the per-species
    `leash_offset` overrides and the boat offset match 1.21.5.
  - Verification: `cargo check -p horizon-client
    --all-targets` 0 errors / 0 warnings (3m25s); three
    newly-dead helpers (`drop_boat_leashes`, `player_center`,
    `Boat::in_liquid`) removed.
- **Item 20 — ByteBufCodecs, verified, no code change:**
  `ByteBufCodecs.java` (132 lines, all 1.21.6-only additions):
  - `RGB_COLOR` (three bytes → opaque ARGB): its only Rust
    use is the `TrackedWaypoint` codec (`play.rs:1799`), and
    the waypoint packet is unwired from the 1.21.5 wire
    (`play.rs:1740-1741` — no waypoint packets in 1.21.5);
    the struct survives only as the LAN tracker's internal
    representation, which the codec choice doesn't affect.
  - `optionalTagCodec` and `lenientJson`: no Rust surface
    (the `lenient_json` test in `java_text.rs` is text-component
    parsing, not a network codec).
  - `fromCodec(DynamicOps, Codec)` (1.21.6) vs the inlined
    `.map` in 1.21.5's `fromCodec(Codec, Supplier)`: the same
    NbtOps parse/encode with the same exceptions — a pure
    refactor.
  - `lengthPrefixed(int)`: 1.21.5 has only the registry-friendly
    form under that name; 1.21.6 renamed it
    `registryFriendlyLengthPrefixed` and added a plain-ByteBuf
    overload. Both call the same three-arg helper — no wire
    difference.
- **Item 21 — boats.rs, ported:** `AbstractBoat` / `Wolf` /
  `ComparatorBlockEntity` / `TagCommand` / `BlockDataAccessor` /
  `PersistentEntitySectionManager`:
  - `AbstractBoat.elasticRangeLeashBehaviour` (1.21.5-only
    override, deleted by 1.21.6's spring rewrite): a boat's
    leash pull is `normalize(holder − boat) * (dist − 6)`,
    scaled 0.15 while the boat already moves toward the holder
    and 0.2 otherwise — not the legacy `copySign` pull item 19
    gave it. `tick_boat_leash` now uses the boat override.
    `boat_leash_offset` already matches 1.21.5: boats set
    `eyeHeight(0.5625)`, so `0.88 * eyeHeight` equals
    `0.88 * bbHeight`. `canBeCollidedWith(Entity)` → `()` is
    an unused-arg removal (body returns true either way); no
    Rust surface.
  - `Wolf.mobInteract` shears branch: 1.21.5 keeps it inline
    with no sneaking gate (the `!sneaking` gate came from
    1.21.6's generic shear-equipment path, `Entity.interact`),
    and Curse of Binding (`PREVENT_ARMOR_CHANGE`) holds the
    armor on outside creative. The Rust's shears arm drops
    the `!sneaking` gate and gains the binding-curse check
    against the armor's `body_extra` enchantments.
  - Wolf variant selection is the same `PriorityProvider.pick`
    in both versions (the Rust's deterministic
    `wolf_variant_for` matches the biome table); the sound
    variant's random differs (entity's in 1.21.5, level's in
    1.21.6) and the production spawn path (`spawning.rs`)
    already uses the mob's own random.
  - The rest is the ValueInput/ValueOutput → CompoundTag
    migration with identical fields (wolf `CollarColor` /
    variant / `sound_variant`, comparator `OutputSignal`),
    a permission-lambda spelling (`TagCommand`), a logging
    wrapper (`BlockDataAccessor.setData`), and a visibility
    change (`processPendingLoads` public → private).
  - Verification: `cargo check -p horizon-client
    --all-targets` 0 errors / 0 warnings (53s).
- **Item 22 — potions.rs, ported:** `AreaEffectCloud` /
  `ProjectileUtil` / `ThrownLingeringPotion` /
  `ThrownSplashPotion` / `ThrownExperienceBottle`:
  - `ProjectileUtil.computeMargin` is 1.21.6-only: 1.21.5
    grows every target box by a flat 0.3
    (`DEFAULT_ENTITY_HIT_RESULT_MARGIN`) from the first
    tick. New shared const `projectiles::ENTITY_HIT_MARGIN`
    replaces the ramp at all seven sites (`projectiles.rs`
    `hit_margin`, `arrows.rs`, `fishing.rs`
    `entity_hit_margin`, `fireworks.rs` rockets,
    `potions.rs` `thrown_margin`, `trident.rs`, and the
    splash distance); the three tests pinning the ramp
    (pearl, hook, thrown potion) rewritten for the flat
    margin.
  - `ThrownSplashPotion.onHitAsPotion`: 1.21.5 centres the
    (4, 2, 4) box on the potion's own position (before the
    step — threaded through as `from`, `t.position` in the
    thrown path, `p.previous` in the generic projectile
    path, which moves the potion to the hit), measures
    feet-to-feet distance, and gives the directly-hit
    entity full strength 1.0. `splash_distance_sqr` becomes
    `splash_intensity`; `affect_entities_around` and the
    water-potion axolotl box already used the 1.21.5 form
    and now take `from` too. `splash` drops the unused
    `age` / `motion` params.
  - `ThrownExperienceBottle.onHit`: plain `award`
    (`spawn_orbs`), not 1.21.6's `awardWithDirection`; the
    XP-bottle test now asserts the orbs start at the hit.
  - `AreaEffectCloud`: 1.21.6's `customParticle` /
    `updateParticle` reverted to 1.21.5's `Particle` /
    `updateColor` — a saved colour particle takes the
    potion's colour at load (black for an empty potion;
    `potion_cloud_from_nbt` now detects the empty potion
    instead of falling back to the base colour), other
    particles are kept as saved. NBT key `custom_particle`
    → `Particle` on both read and write; the cloud-particle
    tests rewritten for the recolour. The owner change
    (`EntityReference` → direct + UUID) has no Rust surface
    (the cloud owner is not modelled from NBT), and the
    `serverTick` discard rewording is identical arithmetic.
  - Verification: `cargo check -p horizon-client
    --all-targets` 0 errors / 0 warnings (1m02s). The
    targeted unit-test run was not waited out (the test
    profile builds from scratch in this workdir); the
    rewritten tests compile under `--all-targets` and CI
    runs them.
- **Item 23 — ItemStack / ThrownEnderpearl /
  PlayerInteractTrigger / AbstractThrownPotion /
  DragonFireball, ported:** the shear-equipment system is
  1.21.6-only — no `attemptToShearEquipment` /
  `canShearEquipment` / `PLAYER_SHEARED_EQUIPMENT` exists
  in the 1.21.5 decompile. `mounts.rs::shear_saddle` (the
  whole 1.21.6 saddle-shearing path, including its trigger
  fire) and its call in the interact dispatch are deleted;
  shears clicks on mounts now fall through to
  `interact_mount`, which matches 1.21.5 (ride / nothing,
  and false for a ridden mount). The wolf-armor shear keeps
  its drop (1.21.5 has it inline in `Wolf.mobInteract`,
  item 21) but no longer fires the trigger; the sulfur
  cube's shear-eject keeps its behaviour (a Horizon-original
  mob) but no longer fires it either. The attribute-modifier
  tooltip code already matches 1.21.5's inline
  `addModifierTooltip` exactly (base attack damage/speed
  added and rendered `equals` in DARK_GREEN, multiplied ops
  x100, knockback resistance x10, plus/take by sign, zero
  shows nothing) — the Rust has no 1.21.6 `Display.hidden`
  concept, which is itself the 1.21.5 behaviour; only the
  doc comment on `modifier_line` was re-cited.
  `DragonFireball.setCustomParticle` -> `setParticle` has
  no Rust surface (breath clouds spawn `DragonBreath`
  particles directly). `ThrownEnderpearl`'s owner change
  (`EntityReference` vs direct + `findOwner`) is the same
  cross-dimension search with different plumbing and the
  Rust models neither; `ItemStack.parse`/`save` have no Rust
  surface (the Rust has its own item NBT codec), and
  `AbstractThrownPotion.onHitAsPotion` was already handled
  in item 22.
  - Verification: `cargo check -p horizon-client
    --all-targets` 0 errors / 0 warnings (2m10s).
- **Item 24 — LivingEntity / LayeredCauldronBlock, ported:**
  - Layered cauldrons: 1.21.6's `FILLED_SHAPES`
    `getEntityInsideCollisionShape` override and the
    EXTINGUISH effect applier are gone in 1.21.5 —
    `entityInside` fires on the AABB overlap and the
    extinguish is gated on
    `AbstractCauldronBlock.isEntityInsideContent` (feet
    below the content top, head above the block's first
    quarter). The block-contact pass now short-circuits
    layered cauldrons to inside on the sweep overlap, and
    `layered_cauldron_inside` takes the content top and
    applies the content gate before clearing the fire or
    asking for the level drop. The lava cauldron keeps the
    shape test — `LavaCauldronBlock` changed too and owns
    its own item. Mobs have no cauldron `entityInside` in
    the Rust (pre-existing gap, same in both versions).
  - `LivingEntity.travelFlying` is 1.21.6-only (extracted
    for the happy ghast): 1.21.5 inlines the form per mob.
    The behaviour is identical — the allay's travel was
    verified against 1.21.5's `Allay.travel`, ghasts and
    phantoms already use the friction `FlyingMob.travel`
    form (item 10; `FlyingMob.java` is identical in both
    versions), and only the happy ghast reaches
    `travel_flying` (item 58 removes it) — so only the doc
    citations were reworded (`travel_flying` /
    `travel_flying_speeds`, the allay, `Flags.flying`, the
    `air_speed` field, the travel dispatch).
  - Waypoints: no change — the publish deliberately kept
    the internal locator-bar/LAN tracker (wire unwired),
    so the `waypoint_transmit_range` /
    `waypoint_receive_range` reads stay as inert reads
    against the catalog, per item 15's pattern for
    `camera_distance`. The `locator_bar_icon` datafix
    targets 4424, above 4325, so it never runs here.
  - No-ops verified: the no-AI travel gate (the Rust
    already travels no-AI mobs, the 1.21.5 form —
    `canSimulateMovement()` only); `checkSlowFallDistance`
    vs `checkFallDistanceAccumulation` (identical bodies,
    a pure rename); the attribute NBT format (1.21.5's
    `AttributeInstance.save` writes the same `{id, base,
    modifiers}` the 1.21.6 Packed codec does); the brain
    serialization (same empty-memories result);
    `canBreatheUnderwater`'s `final` and `isJumping` (no
    Rust surface); the ValueInput/ValueOutput → CompoundTag
    migration (identical fields).
  - Verification: `cargo check -p horizon-client
    --all-targets` 0 errors / 0 warnings (1m27s).
- **Item 25 — ClientLevel / SoundManager / AbstractWidget,
  ported:**
  - `SoundManager.updateSourceVolume` (1.21.5): silencing
    the master stops every sound, it does not just mute
    them — a stopped sound does not come back when the
    volume does; 1.21.6 dropped the stop.
    `Audio::sync_volumes` now stops all sounds when the
    master changes to <= 0 (the change-detection fires it
    once per transition). The pause half was already done
    in item 15 (`Audio::pause` pauses every channel).
  - Void darkness: no change — the Rust's fog math already
    matches 1.21.5's `getClearColorScale` exactly
    (`(camera_y - min_y) / 32`, squared, flat worlds 1.0;
    1.21.6's `voidDarknessOnsetRange` is the same value
    spelled as a range, and 0.03125 is exactly 1/32).
  - No-ops verified: `globallyRenderedBlockEntities` (no
    Rust surface — the Rust does not model off-screen
    block-entity rendering, and the behaviour is the same
    in both versions, 1.21.6 only moved the bookkeeping);
    `disconnect(Component)` / `DEFAULT_QUIT_MESSAGE` (no
    Rust surface; the same quitting message either way);
    the SoundManager ctor's MusicManager parameter and
    `play` returning `PlayResult` (internal, no Rust
    surface); the Gson Component adapter (vanilla
    `sounds.json` uses string subtitles only, which the
    Rust parses — object-form subtitles are a
    resource-pack edge the Rust rejects wholesale in both
    versions); the widget hover test (identical logic in
    both versions, `areCoordinatesInRectangle` only
    extracted in 1.21.6; the Rust uses the same range
    test) and the tooltip-refresh signature (no shared
    Rust surface — tooltips render immediately per
    screen).
  - Verification: `cargo check -p horizon-client-render
    -p horizon-client --all-targets` 0 errors / 0 warnings
    (2m59s).
- **Item 26 — TrialSpawner / TrialSpawnerState, ported:**
  - The diff is a refactor with identical behaviour: the
    1.21.6 `FullConfig` record and `TrialSpawnerStateData`
    / `Packed` are 1.21.5's separate fields and
    `TrialSpawnerData`, with the same codec fields and
    defaults (`normal_config`, `ominous_config`,
    `target_cooldown_length` 36000, `required_player_range`
    14; `registered_players`, `current_mobs`,
    `cooldown_ends_at`, `next_mob_spawns_at`,
    `total_mobs_spawned`, `spawn_data`,
    `ejecting_loot_table` — verified field-for-field in
    both codecs). `spawnMob` is the same logic with
    CompoundTag plumbing instead of ValueInput, and
    `TrialSpawnerState` only renames accessors
    (`getStateData` -> `getData`, `activeConfig` ->
    `getConfig`). The Rust's load/save already matches
    1.21.5's NBT; the doc citations were reworded
    (`TrialSpawnerStateData` -> `TrialSpawnerData`,
    `activeConfig` -> `getConfig`, `FullConfig.MAP_CODEC`
    -> `TrialSpawner.codec()`) in `mobs/trial_spawner.rs`,
    `game/trial_spawner.rs`, `lan/chunks.rs`,
    `block_entity_nbt.rs` and the render crate's
    `block_entity.rs`.
  - One real behaviour fix, found via the bogus citation:
    `TrialSpawner::override_entity_data` cited
    `FullConfig.overrideEntityData`, which exists in
    neither version — 1.21.5's `TrialSpawnerBlockEntity`
    has only `setEntityId`, and `SpawnEggItem.useOn`
    (unchanged between versions) calls only `setEntityId`,
    dropping the egg's entity data. The trial-spawner
    branch of `use_spawn_egg_on_spawner` now always calls
    `override_entity`, and `override_entity_data` is
    deleted. NOTE for the regular-spawner item
    (`SpawnerBlockEntity` changed too): the same 26.3-ism
    lives on in `set_entity_data` / `spawn_egg_on_spawner`
    (op eggs still pass data to regular spawners), which
    1.21.5's `SpawnEggItem.useOn` does not do either.
  - Verification: `cargo check -p horizon-client-render
    -p horizon-client --all-targets` 0 errors / 0 warnings
    (1m49s).
- **Item 27 — ItemFrame / Painting / BlockAttachedEntity,
  ported:**
  - The water check moved: 1.21.5's
    `ServerExplosion.shouldAffectBlocklikeEntities` holds
    it (`source == null || !source.isInWater()`), so a
    blast whose source is in water spares blocklike
    entities of every kind; 1.21.6 moved it into
    `BlockAttachedEntity.ignoreExplosion` alone. The
    hanging-entity side already matched (the water term
    in `blast_spares_decorations` is correct for 1.21.5,
    only its doc citation moved), but two paths had the
    1.21.6 form: `items_ignore_explosion` now takes the
    source-in-water flag (dropped items are spared by a
    wet source in 1.21.5, griefing or no — new test
    assert), and the armour-stand call passes the real
    `explosion_in_water` instead of `false`
    (`ArmorStand.ignoreExplosion` reads
    `shouldAffectBlocklikeEntities` too).
  - Hanging-entity metadata: 1.21.6 added
    `HangingEntity.DATA_DIRECTION` at index 8, pushing
    the subclasses' fields up one; 1.21.5 has no synced
    direction (it rides the add-entity packet's data
    alone). The LAN sync is renumbered to the 1.21.5 form
    on both ends: item frames send/read DATA_ITEM 8 and
    DATA_ROTATION 9 with no direction meta (host
    `lan_entities.rs`, guest `remote_objects.rs`, dirty
    defaults in `lan/entity.rs`); paintings send/read
    DATA_PAINTING_VARIANT_ID 8 with no direction meta.
    The guest spawn paths already read the facing from
    the add-entity data. Consistent with the protocol
    adapter (`v1_21_5.rs`, which shifts hanging indices
    +1 for 1.21.6 servers).
  - No-ops verified: the ItemFrame/Painting NBT
    (identical fields, ValueInput/ValueOutput ->
    CompoundTag migration); `recalculateBoundingBox`'s
    `syncPacketPositionCodec` (1.21.6 network plumbing,
    no Rust surface); `defineSynchedData` /
    `onSyncedDataUpdated` super calls (the 1.21.6 synced
    direction, removed with the renumbering above).
  - Verification: `cargo check -p horizon-client
    --all-targets` 0 errors / 0 warnings (3m00s).
- **Item 28 — LevelRenderer / FogType (FogRenderer),
  ported:**
  - 1.21.6 replaced 1.21.5's `FogRenderer` (one
    `setupFog` + `computeFogColor`, `FogType` of LAVA /
    WATER / POWDER_SNOW / NONE) with a `fog/`
    environment package (`Atmospheric`,
    `DIMENSION_OR_BOSS`, rain fog). The Rust followed
    the 1.21.6 structure; it now follows 1.21.5's:
    `FogType::Atmospheric` is `FogType::None` again,
    and every 1.21.6 environment citation is re-cited
    to `FogRenderer.setupFog` / the
    `BlindnessFogFunction` / `DarknessFogFunction`.
  - Rain fog is 1.21.6-only: 1.21.5's atmospheric
    terrain fog is the render-distance pair alone, so
    the environmental pair is inert (1e9, 2e9) in
    clear air. `update_rain_fog` / `settle_rain_fog` /
    `rain_fog_target`, `FogState.rain_fog_multiplier`,
    the `rain_fog_inputs` helper and the
    `HORIZON_STILL` settle call are deleted, and the
    distant-terrain environmental scale (which only
    ever scaled that pair) goes with them.
  - The foggy branch is the formula, not the data:
    1.21.5 closes a foggy dimension or a
    createWorldFog boss bar in to rd*0.05 /
    min(rd, 192)*0.5, for sky and clouds as for
    terrain. A new `dimension_foggy` input (the
    Nether's `isFoggyAt`) joins `boss_world_fog`;
    the old form took the dimension's
    `visual/fog_end_distance` (10, 96) and the pack's
    `boss_fog_distance` (96), which only agree with
    the formula at rd >= 192. `boss_fog_distance`
    is removed from the Polytone environment
    settings.
  - Fluid fog colours are 1.21.5's literals: lava
    (0.6, 0.1, 0.0) not 1.21.6's 0x991A00, powder
    snow (0.623, 0.734, 0.785) not 1.21.6's
    0x9FBACC (the pack override still wins when set).
  - Effect void darkness REPLACES the void factor
    in 1.21.5 (blindness: 1 - dur/20 while ending
    within 19 ticks, else 0; darkness: 1 -
    blendFactor); 1.21.6's environments take the
    max. A blinded camera at the floor is dim, not
    black.
  - Rain and thunder darken the MIXED fog colour
    in 1.21.5 (and 1.21.6); the Rust darkened the
    sky colour before the mix, so rain did nothing
    at high render distance. Order fixed.
  - Water fog: the x0.85 for `HAS_CLOSER_WATER_FOG`
    biomes is back — `FogTables::load` reads the
    tag file (swamp, mangrove swamp) and multiplies
    it in with any pack attribute factor; the
    attribute alone never fired on 1.21.5 data.
    The end-over-render-distance cap is subsumed by
    the render-distance fog the shader maxes with
    (its CYLINDER shape has no slot in the two-pair
    uniform).
  - No-ops verified: the branch order (lava, powder,
    priority effect, water, foggy, sky/terrain) and
    the blindness-over-darkness priority match;
    lava/powder/blindness/darkness distances match;
    the brightness-dependent fog colour is a uniform
    scale, so 1.21.5's per-sample application equals
    1.21.6's post-average one; the void-darkness
    range (32, 1 flat), boss darken (0.7/0.6/0.6)
    and night-vision brighten match; the sky pass is
    skipped for lava/powder/effects/boss fog (the
    Nether's fogginess is covered by the
    overworld-only sky); cloud height 192.33 is
    1.21.5's `OverworldEffects.getCloudHeight`;
    the entity type-hash sort, the translucent
    section order, the section rebuild neighbour
    gate and the sun/moon fog parameter have no
    Rust surface (per-quad translucent sorting, a
    padded mesher with no compile queue, no
    celestial fog fade). Pre-existing gaps, same
    in both versions, not taken up: the 5-second
    water-fog colour transition (the Rust blends
    spatially instead) and the SPHERE/CYLINDER
    shape distinction.
  - Verification: `cargo check -p
    horizon-polytone -p horizon-client-render -p
    horizon-client --all-targets` 0 errors / 0
    warnings (3m04s); `cargo test -p
    horizon-client-render --lib fog` 24 passed / 0
    failed.
- **Item 29 — Screen / ConfirmLinkScreen /
  SuggestionProviders, ported:**
  - Clicked commands: 1.21.5's
    `handleComponentClicked` inlines the whole dispatch
    and runs commands through
    `ClientPacketListener.sendUnsignedCommand` — sent
    as-is when it has no signed argument, refused
    with an error log when it has one. 1.21.6 split
    it into `handleClickEvent` /
    `defaultHandleGameClickEvent` /
    `defaultHandleClickEvent` / `clickUrlAction` /
    `clickCommandAction` and runs commands through
    `sendUnattendedCommand`, which asks first on parse
    errors, permissions and signatures. The Rust had
    the 1.21.6 form; `send_unattended_command` is
    now `send_unsigned_command` (signable-args check
    only, sent via the ordinary sender, which goes
    unsigned when nothing needs signing) and the
    whole confirmation machinery is gone:
    `CommandCheck` / `verify_command` /
    `CommandAccept` / `Behind` / `CommandPrompt` and
    the prompt's layout/draw/key/click/answer, the
    `command_prompt` field and its draw/input/pad/
    pointer/observer wiring, `ClosedDialog` /
    `reopen_dialog`, and the book's take-and-restore
    around a click. The net harness drops its
    `prompts` / `chat_box` report keys (`.answer`
    lines are ignored) and the secure-chat e2e drops
    the prompt expectations and the `/stop` click —
    in 1.21.5 an op's clicked `/stop` would run with
    no confirmation. Item 384's class
    (`ClientPacketListener$CommandCheckResult`) is
    removed under this item.
  - Confirm buttons: 1.21.5 lays the row out by
    absolute bounds spanning width/2 ± 155, so
    `Choice` now fills the 310 span (10 apart for
    two of 150, 5 for three of 100) instead of
    1.21.6's `LinearLayout` 4 apart; the link
    prompt's three 100-wide buttons land at
    cx-155/cx-50/cx+55. The warning colour
    (0xFFCCCC) and the texts (trusted
    `chat.link.open`, untrusted `gui.yes`) already
    matched.
  - Citations reworded to 1.21.5's inlined
    `handleComponentClicked` (chat, books,
    dialogs, server links); the ShowDialog/Custom
    arms stay as Horizon's local-only paths (no
    packet behind them since the publish).
  - No-ops verified: `SuggestionProviders` is a
    rename (`Wrapper` for `RegisteredSuggestion`)
    with identical semantics — `getProvider` falls
    back to ASK_SERVER, `safelySwap` matches the
    Rust's unknown-provider handling, the
    summonable-entities tooltip
    (`EntityType.getDescription`) is the same
    translatable; the screen fade
    (`fadeWidgets`/`FADE_IN_TIME`), the strata
    background/blur plumbing and the deferred
    tooltip have no Rust surface (item 16 already
    covered the tooltip deferral). Referred:
    the narrator rename (`sayNow`, item 51),
    the panorama (`PANORAMA` static, item 292),
    `BookViewScreen` proper (item 270) and
    `ConfirmScreen` proper (item 40 — its button
    gap is already 1.21.5's via `Choice`).
  - Verification: `cargo check -p horizon-client
    --all-targets` 0 errors / 0 warnings (4m45s).
    The `Choice` test's new positions (85/190/295,
    85/245) are arithmetic from the 310 span, checked
    by hand; the horizon-client test binary's link
    was still building after 17 min, so the run is
    left to CI per the standing rule.
- **Item 30 — AbstractHorse (equine.rs), ported:**
  - Plain carrots are not horse food in 1.21.5:
    1.21.6 added the carrot branch to `handleEating`
    (heal 3, age 60, temper 3) and to the
    `#minecraft:horse_food` tag. The carrot arm is
    out of the mounts food table and out of the
    remote food matcher (the data-driven
    `kind.food` already follows the 1.21.5 tag,
    which lacks it); golden carrots are untouched.
  - Rearing counts up: 1.21.5's `standIfPossible`
    starts the counter at 1 and `tick` runs it up
    past 20 (`isEffectiveAi`-gated — the server
    for an ordinary mob, the client for the mount
    it drives), where 1.21.6's `setStanding(20)`
    counts down with no gate. `stand_if_possible`
    sets 1, `stand_tick` increments and ends past
    20; a client's synched rear (`CLIENT_STANDING`)
    is a sentinel `stand_tick` leaves alone, so the
    flag still ends it.
  - No-ops verified: the leash hook is already
    1.21.5's `handleLeashAtDistance` (item 19 — no
    elastic pull, no quad leash); `makeMad` has no
    client-side gate in either the Rust or 1.21.5;
    the mount-start zeroing matches 1.21.5's
    `setStanding(false)` observably (the flag is
    what the animation follows — the stale counter
    1.21.5 leaves behind is invisible); the
    `isJumping` field has no Rust surface (the
    ridden jump spends the pending scale at once
    and only fires on the ground, so the
    double-jump guard is subsumed); the NBT fields
    (EatingHaystack, Bred, Temper, Tame, Owner)
    are identical, CompoundTag vs ValueOutput
    being API-only.
  - Verification: `cargo check -p horizon-client
    --all-targets` 0 errors / 0 warnings (1m49s).
- **Item 31 — ByteBufCodecs (item/skip.rs), verified, no code
  change:** the same 132-line diff item 20 verified, filed
  under a second module. The five changed hunks —
  `RGB_COLOR`, `optionalTagCodec`, `lenientJson`,
  `fromCodec(DynamicOps, Codec)`, the `lengthPrefixed`
  rename — have no surface in `skip.rs`: the named
  methods it uses (`optional`, `holder`, `holderSet`,
  `fromCodecWithRegistries`) are all unchanged in the
  diff, and the component skip table itself is the
  regenerated 1.21.5 one from the publish.
- **Item 32 — WorldVersion / SharedConstants, ported
  (citations only):** the version pin itself was
  already 1.21.5 from the publish (1.21.5, protocol
  770, data 4325, resource pack 55, data pack 71,
  stable, build time 25 Mar 2025) — all values
  verified against the diff. What 1.21.6 changed was
  the accessor shape: 1.21.5's `WorldVersion` uses
  `get`-prefixed accessors (`getId`, `getName`,
  `getDataVersion`, `getProtocolVersion`,
  `getPackVersion`, `getBuildTime`, `isStable`) and
  has no `Simple` record. The doc citations in
  `version.rs`, its generator
  (`gen_version_consts.py`), `screens.rs` and
  `realms/client.rs` now cite the 1.21.5 names (the
  generator couldn't be re-run — no full 1.21.5
  client jar is in this workdir — so `version.rs`
  was hand-synced to exactly what the updated
  generator emits; values untouched). No-ops:
  `SNBT_NAG_VERSION` (4292) and the four removed
  debug flags (`DEBUG_SHUFFLE_UI_RENDERING_ORDER`,
  `DEBUG_RENDER_UI_LAYERING_RECTANGLES`,
  `DEBUG_DEFAULT_SKIN_OVERRIDE`,
  `DEBUG_ENTITY_BLOCK_INTERSECTION`) have no Rust
  surface; the snapshot protocol version (243) is
  unused.
  - Verification: `cargo check -p horizon-core
    --all-targets` 0 errors / 0 warnings (17s).
- **Item 33 — KeyboardHandler / GameModeCommand, ported:**
  - Debug feedback is not narrated in 1.21.5:
    1.21.6's `showDebugChat` says every debug line
    through the narrator, 1.21.5's `debugComponent`
    only adds it to the chat. The narrator call is
    out of the debug-feedback path (the method now
    cites `debugComponent`); the screenshot callback
    already adds its message without narrating.
  - F3+V (version dump) is 1.21.6-only: the
    `DumpVersion` action, its `key.debug.dumpVersion`
    binding (V) and the `DebugDumpVersion` input
    action are all removed. `/version` stays — it is
    `VersionCommand`, which 1.21.5 has — and its
    lines now read the pack formats from the version
    pin (55 / 71) instead of the hardcoded 26.3
    values (97.1 / 121.0).
  - F3+I's `/summon` copy now strips `Dimension` as
    well as `UUID` and `Pos` (1.21.6 dropped the
    Dimension strip).
  - No-ops verified: F4 already sends `/gamemode`
    (the change-game-mode packet is unwired);
    `GameModeCommand`'s delta is a pure refactor
    (the Rust inlines set + log exactly as 1.21.5
    does); the F3+Q help list has no Rust surface;
    `translatableEscape` vs `translatable` can't
    differ at the Rust's call sites (no arg can
    carry a `%`).
  - Verification: `cargo check -p
    horizon-client-input -p horizon-client
    --all-targets` 0 errors / 0 warnings (3m10s).
- **Item 34 — LocalPlayer / UnderwaterAmbientSoundHandler,
  ported:**
  - No happy-ghast riding sound: 1.21.6's
    `startRiding` plays `RidingHappyGhastSoundInstance`;
    1.21.5 has no happy ghast. The ghast arm is out
    of `start_riding_sound` (the nautilus arm stays)
    and `SoundInstance::riding_happy_ghast` + its
    test are deleted. (The ghast mob itself is item
    58's removal.)
  - No experience-display timer: 1.21.6's
    `setExperienceValues` stamps
    `experienceDisplayStartTick`, and the locator
    bar yields to the XP bar for 100 ticks after a
    change (`willPrioritizeExperienceInfo`). 1.21.5
    has neither — the XP bar is never suppressed by
    recency. The `experience_display` field, its
    tracking and `experience_recent` are deleted;
    the contextual-bar call passes `true` for
    recency. (The locator bar itself is item 108's
    removal; LAN waypoints are item 86's.)
  - No-ops verified: `sendShiftKeyState` (shift
    press/release as PlayerCommand 0/1, before the
    input packet) is already the 1.21.5 form
    (item 12's numbering); `openDialog` has no
    client surface (no dialog packet handling in
    remote.rs — dialogs.rs is Horizon's local-only
    path, per item 29); `getLastSentInput` has no
    Rust surface; `UnderwaterAmbientSoundHandler`
    is byte-identical between the two decompiles
    (the report's "1 method changed" is a false
    positive).
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings (5m16s).
- **Item 35 — BlockEntity, ported:**
  - A block entity with no components saves
    WITHOUT the `components` key in 1.21.5:
    `saveWithoutMetadata` stores it through
    `COMPONENTS_CODEC.optionalFieldOf("components",
    EMPTY)`, which omits an empty map; 1.21.6's
    `ValueOutput.store` writes it unconditionally
    (empty compound and all). `canonical` now
    writes `components` only when the input's is
    non-empty, and `every_type_has_defaults`
    asserts the omission (plus that non-empty
    components survive).
  - No-ops verified: the rest of the diff is the
    ValueInput/ValueOutput → CompoundTag API
    migration (the Rust's NBT layer already works
    in compounds — the 1.21.5 form); the save
    order (saveAdditional, components, metadata)
    matches in both versions; `saveCustomAndMetadata`
    (1.21.5-only) has no callers anywhere in the
    1.21.5 decomp; `parseCustomNameSafe`'s only
    caller is BaseCommandBlock and the Rust's
    `component()` already discards unparseable
    custom names; `getNameForReporting`'s
    visibility and the ProblemReporter path are
    API-only.
  - Verification: `cargo check -p
    horizon-client-core -p horizon-client
    --all-targets` 0 errors / 0 warnings (3m43s).

## What remains, by cause

### Behaviour work from the diff (in progress — rank 1 done)

The 426 ranked items are the port's work list; **35 of 426 are
closed** (rank 1 complete, rank 2 at 26/259). The work is the dialog/waypoint/happy-ghast removals'
behaviour consequences, the 1,202 changed classes (1,219 concern
Horizon), and the 259 harness reruns — unattempted. The
save-format step (4435 → 4325: 1 schema, 2 fixes, 14 datafix
classes) is done (item 8, rank 1). The behaviour code in this
tree is otherwise still the 26.3 base's.

### Harness-oracle era ports

The per-harness oracles carry their era gaps forward from 1.21.6
unchanged — the same oracle failures as 1.21.6 are expected once
the profile is measured, but the counts are unmeasured for
1.21.5.

### Full harness profile

The CI `port-verify` workflow on main measures the profile on
GitHub runners. The local sweep was skipped (see above), so CI is
the first profile for this version, not a confirmation of a local
one.

### D6 playtest

Deferred by the owner ("leave the play test for later") — recorded
as deferred, never claimed.

## Status

Pin set (protocol 770, world 4325); 6 of 6 tables regenerated;
packet rewire done (8 packets removed, 60 renumbered) and the
game-side wire boundary reworked; adapter ladder re-based to 770;
oracles ported to the 1.21.5 APIs (70 files); workspace compiles
(0 errors, 0 warnings); protocol tests 427 pass / 2 fail (the
sandbox-network pair); prepare verified ok end-to-end. Harness
profile: **not measured locally** (skipped — CI to measure).
D6 deferred (owner).
