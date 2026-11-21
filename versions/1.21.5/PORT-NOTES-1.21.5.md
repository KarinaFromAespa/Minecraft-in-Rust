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
    action are all removed. (`/version` itself is
    also 1.21.6-only — corrected under item 36,
    which removes the command outright; the
    `version_lines` pack-format fix made here was
    superseded by that removal.)
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
- **Item 36 — Commands / Fill / SetBlock / TagLoader,
  ported:**
  - `/version` does not exist in 1.21.5
    (`VersionCommand` has no class and no register
    call in the 1.21.5 decomp — it arrived in
    1.21.6): the tree literal, the dispatch arm,
    `cmd_version` + `version_lines`, the
    `OPEN_WHEN_INTEGRATED` entry and the dedicated
    test's version cases are all removed. (This
    corrects item 33's note, which kept `/version`.)
    `/dialog` and `/waypoint` were already absent
    from the tree (publish).
  - `/setblock` and `/fill` notify neighbours with
    plain `updateNeighborsAt(pos, newBlock)` in
    1.21.5; 1.21.6's `updateNeighboursOnBlockSet`
    also ran the replaced block's
    `affectNeighborsAfterRemoval` and the
    analog-output update. Both commands now call a
    plain `update_neighbors_at` helper; fill
    records positions only (no old states); the
    `update_neighbours_on_block_set` helpers are
    deleted from commands.rs and level/mod.rs.
  - The commands packet has no restricted flag in
    1.21.5 (flags are executable 4, redirect 8,
    suggestions 16; bit 32 is 1.21.6's
    `FLAG_RESTRICTED`): the field is out of the
    protocol codec, the LAN tree builder, the
    client tree model (`Node.restricted` and
    `without_restricted`, 1.21.6-only) and all
    literals/tests. The tree itself still comes
    from the regenerated 1.21.5 commands.json.
  - No-ops verified: the `hasPermission` lambdas
    vs `PermissionCheck` wrapper are identical
    semantics (the Rust gates on level numbers);
    `trimOptionalPrefix`/`COMMAND_PREFIX`,
    `DataPackCommand`/`DebugConfigCommand`
    register signatures and the
    AbstractMinecartContainer NBT change are
    API-only (the minecarts comment now cites
    CompoundTag's getters).
  - KNOWN GAP: TagLoader reads tag files with
    Gson's lenient `JsonParser` in 1.21.5
    (`StrictJsonParser` in 1.21.6); the Rust's tag
    codec is serde_json throughout (strict). Only
    malformed data-pack tag files differ (comments,
    single quotes, unquoted strings) — vanilla's
    own tag files are strict JSON. A lenient
    fallback needs a bridge in shared worldgen
    tag code; not done here.
  - Verification: `cargo check -p
    horizon-command -p horizon-protocol -p
    horizon-world -p horizon-client --all-targets`
    0 errors / 0 warnings (13m48s).
- **Item 37 — Music / MusicManager, ported:**
  - The music frequency option is 1.21.6-only
    (`MusicFrequency` DEFAULT/FREQUENT/CONSTANT,
    `musicFrequency` in Options, `getNextSongDelay`
    capping the delay at `id * 1200`): the enum,
    `set_frequency`, the `frequency` field and the
    `musicFrequency` option surface are all
    removed (music.rs, game/music.rs, game.rs,
    options_file.rs, settings.rs, options_ui.rs).
    The delay is 1.21.5's plain
    `Mth.nextInt(minDelay, maxDelay)` on track
    end (and on the nothing-to-play retry), with
    the outer `min(nextSongDelay, maxDelay)`.
  - The Now Playing toast is 1.21.6-only
    (`NowPlayingToast`, `showNowPlayingToast` /
    `hideNowPlayingToast`, `toastShown`, the
    `musicToast` option, the `showNowPlayingToast`
    → `musicToast` options fixer): `toast_shown`,
    `toast_requests` and
    `show_now_playing_toast_if_needed` are out of
    the manager, `NowPlayingToast` / `MusicNotes` /
    `draw_now_playing` are out of toasts.rs, the
    `now_playing` / `music_toast_state` /
    `music_notes` fields and the four
    `draw_music_toast` calls are out of game.rs,
    and the toast kind is out of the
    `ToastAddEvent` doc. `startPlaying` is
    1.21.5's form: play the track and set its
    volume when its sound is not the empty sound —
    no PlayResult switch, no toast.
  - `stopPlaying()` is 1.21.5's form: stop the
    track and `nextSongDelay += 100` only — no
    frequency draw, no toast hide. The
    `situational` parameter is out of
    `stop_playing` / `stop_playing_music`
    (callers in game.rs, remote.rs, win.rs and
    the fixture updated); the old code drew a
    fresh frequency delay AND added 100.
  - The music parity harness is ported to 1.21.5
    too: the oracle's `Mc` now returns a
    `MusicInfo(music, volume)` from
    `getSituationalMusic` (1.21.5 has no
    `getMusicVolume` — the volume rides in the
    info) and the `Toasts` subclass, the
    `gameMusicFrequency` field set and the
    `freq` / `toast` commands are gone; the
    driver drops the `freq` / `toast` script
    lines and the `start` seed's frequency
    argument; the fixture drops the matching
    arms and the `toastShown` output line. The
    options parity harness drops `musicToast` /
    `musicFrequency` from `ALL_KEYS` and uses
    `gamma:0.5` as the load-abort canary tail
    (its `showNowPlayingToast` legacy bodies
    stay: they now exercise unknown-key handling
    on both sides).
  - No-ops verified: `Music`'s codec is identical
    in both versions (the record-vs-class shape
    is API-only — the Rust struct is unchanged);
    the replace branch (`nextInt(0, minDelay /
    2)`) and the `min(nextSongDelay, maxDelay)`
    outer line were already 1.21.5-form.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings (5m19s).
- **Item 38 — ConduitBlockEntity, ported:**
  - The destroy target is 1.21.5's direct
    entity + pending UUID pair, not 1.21.6's
    `EntityReference`: `ConduitServer` gains a
    `pending` slot (`destroyTargetUUID`) and
    `conduit_take_pending` moves the block
    entity's loaded `Target` into it (the block
    entity then holds the resolved target's UUID
    only — what `saveAdditional` and the update
    packet carry — and no block update is sent
    for the move). `conduit_attack` is 1.21.5's
    `updateDestroyTarget`: the resolved target
    stays while alive and within 8 blocks; a
    pending UUID resolves first, an unresolvable
    one is dropped WITHOUT a block update and a
    new target is only picked on the next check;
    while the conduit does not hunt the resolved
    target is dropped but the pending UUID is
    kept for the next hunt (the old code dropped
    it and sent an update on the clear).
  - The client resolves the target from the
    block entity's `Target` UUID
    (`updateClientTarget`, 1.21.5-only) where
    there is no server entry — a client of a
    server, or a conduit in a chunk that does
    not tick blocks: the Rust never drew the
    conduit's beam particles there before (the
    target came from `conduit_servers` only).
    The UUID resolves against the loaded mobs
    within the destroy range; an unresolvable
    one is dropped, not retried, until the next
    update packet — on a client of a server
    only, since on the integrated server the
    block entity's `Target` is the server's
    pending UUID.
  - No-ops verified: the attack (4 magic
    damage + sound, every 40 ticks when active),
    the 8-block keep check, the random pick and
    the destroy-range AABB (unit cube inflated
    by 8) are identical in both versions; the
    ValueInput/ValueOutput → CompoundTag API
    migration is already the Rust's form; the
    `Target` NBT shape (UUID int array) is
    unchanged; `findDestroyTarget`'s exactly-one
    rule is vacuous (UUIDs are unique).
  - KNOWN GAP: a pending UUID in a chunk that
    never ticks blocks stays in the block entity
    until the chunk ticks, so a save in that
    window writes it — 1.21.5's `saveAdditional`
    omits a pending `Target`. The load-to-first-
    tick window (at most 40 ticks) has the same
    gap.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings (2m51s).
- **Item 39 — SignBlockEntity / SignBlock / sign
  editors, ported:**
  - A sign's click commands are `run_command`
    only in 1.21.5: `ClickAction::ShowDialog`
    and `ClickAction::Custom` (and their
    `click_action` arms) are removed from
    signs.rs, the two arms are out of
    `execute_sign_click_commands`, and
    `show_click_dialog` is deleted (its only
    caller was the sign arm). `has_any_click_commands`
    already matched `run_command` only.
    `MinecraftServer.handleCustomClickAction`
    has no Rust surface left.
  - No-ops verified: the sign NBT layout
    (`front_text` / `back_text` with
    messages/color/has_glowing_text,
    `is_waxed`, `allow_op_features`) is
    compound-based already — the
    ValueInput/ValueOutput → CompoundTag
    migration is API-only; the editor's
    offsets (90 standing, 125 hanging), the
    hanging 16x16-at-4.5 background, the
    title at y 40 and the shared text
    highlight are already the 1.21.5 form
    (`offsetSign`'s z and the
    Lighting/flush/RenderType-vs-RenderPipelines
    differences have no wgpu surface); the
    applicator and empty-hand use paths
    already run click commands before the
    waxed-sound / editor branches, as
    1.21.5's `useItemOn` / `useWithoutItem`
    do (the `instanceof ServerLevel` vs
    `isClientSide` guard differs only on a
    path vanilla never calls client-side).
  - NOT this item: the general `ClickEvent`
    model's `ShowDialog` / `Custom` variants
    (text.rs, component.rs) belong to items
    175 (rank 2, `ClickEvent$Action`) and
    391 (rank 3, `ClickEvent$Custom`); the
    `/dialog` command implementation left
    in extra.rs belongs to the dialog item
    (item 36 already took it out of the
    tree).
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings (3m21s).
- **Item 40 — ConfirmScreen / PauseScreen,
  ported:**
  - `Choice` (ConfirmScreen and
    ConfirmLinkScreen) is 1.21.5's manual
    layout, not 1.21.6's LinearLayout column:
    the title sits at `titleTop` (`clamp((h -
    mh) / 2 - 29, 10, 80)`), the message 20
    below it, the buttons 20 below the
    message clamped to `h / 6 + 96 .. h - 24`
    (`Mth.clamp` semantics, so a screen
    shorter than 144 cannot panic); the
    15-row message cap (1.21.6's
    `setMaxRows(15)`) is gone. The button
    row (width/2 ± 155, 10 apart for two of
    150) was already 1.21.5's (item 29). A
    layout test pins the three formulas.
  - The pause menu's custom dialog additions
    are 1.21.6-only (`getCustomAdditions`,
    the `pause_screen_additions` tag, the
    dialog button with
    `CUSTOM_OPTIONS_TOOLTIP`):
    `pause_screen_additions`, the id-14 arm
    and the `additions` parameter are all
    removed; the server links row (id 11,
    opening `ServerLinksScreen`) stays, now
    without the tooltip 1.21.6's dialog
    button carried. The additions test is
    deleted; the split-settings test drops
    its additions cases.
  - No-ops verified: the disconnect flow
    already matches 1.21.5 (external server
    leaves plainly, a local world goes
    through the `menu.savingLevel` screen);
    the disconnect label
    (`menu.returnToMenu` / `menu.disconnect`)
    is the same pair; the Now Playing toast
    on the pause menu is already gone (item
    37); the centered icon button already
    draws its sprite at full opacity
    (1.21.5's `blitSprite` takes no alpha);
    the WorldVersion record → interface
    change is item 32's (done); the
    RenderPipelines → RenderType rename has
    no wgpu surface.
  - NOT this item: the `dialog/` screens
    package (removed in 1.21.5) and the
    `ServerLinksDialogScreen` citations in
    server_links.rs belong to the dialog
    item.
  - Verification: `cargo check -p
    horizon-client --all-targets` 0 errors /
    0 warnings (3m27s).
- **Item 41 — BaseSpawner / spawner and
  command minecarts, ported:**
  - The diff is the ValueInput/ValueOutput →
    CompoundTag migration: 1.21.6 loads the
    spawn entity through a
    `ProblemReporter.ScopedCollector` +
    `TagValueInput`, 1.21.5 reads the
    `CompoundTag` directly, with identical
    logic (spawn position formula, collision
    and spawn-rule checks, `maxNearbyEntities`
    count, finalize-when-unconfigured,
    equipment, level event 2004). The Rust
    was already compound-based and matches
    on every point, as do the save/load keys
    and defaults and the spawner/command
    minecart NBT.
  - Real fix, the item-26 referral: a spawn
    egg used on a REGULAR spawner sets only
    the type in 1.21.5 — `Spawner` has no
    `setEntityData` at all, and
    `SpawnEggItem.useOn` calls only
    `setEntityId`, op or not. The op's
    entity-data merge is removed:
    `set_entity_data` (trait method, impl,
    test) is gone, `spawn_egg_on_spawner`
    returns just the type, and
    `use_spawn_egg_on_spawner` takes no data
    (trial spawners already took only the
    type, item 26). The dispenser call site
    (type selection only) is unchanged in
    behaviour.
  - Verification: `cargo check -p
    horizon-client --all-targets` 0 errors /
    0 warnings (1m32s).
- **Item 42 — ExperienceOrb, ported:**
  - The 1.21.6-only directional spawn is
    gone: `awardWithDirection`,
    `spawn_orbs_toward`/`add_orb_toward`,
    the 0.25 offset along the direction, the
    velocity flip and the spawn-time
    `unstuckIfPossible` (`ExperienceOrb::unstuck`,
    `unstuck_orb` and the unstuck tests are
    all removed). `award`/`spawn_orbs` and
    `add_orb` are 1.21.5's plain forms: the
    orb spawns exactly at the given point,
    its yaw is a `nextDouble` (1.21.6 drew
    a `nextFloat`), and the motion's x/z
    factors are the widened `0.2F`/`0.1F`
    literals. `/summon experience_orb` also
    stops unsticking (the summon constructor
    never did). The XP-bottle test already
    asserted the 1.21.5 form (item 22).
  - The tick's stuck push is unconditional
    in 1.21.5: a colliding orb with no
    player following always gets
    `moveTowardsClosestSpace` (1.21.6 only
    pushes when the next move would still
    collide). The inner collision gate is
    removed.
  - No-ops verified: save/load keys
    (Health/Age/Value/Count) and the merge
    cadence are unchanged;
    `modifyDurabilityToRepairFromXp` takes
    `serverLevel()` in 1.21.5 vs `level()`
    in 1.21.6, the same level for a
    ServerPlayer; `hasImpulse` has no Rust
    surface.
  - Verification: `cargo check -p
    horizon-client --all-targets` 0 errors /
    0 warnings (1m15s).
- **Item 43 — EntityType / SpawnerBlockEntity,
  citations only:**
  - The diff is the ValueInput/ValueOutput →
    CompoundTag migration (`create`, `by`,
    `loadEntityRecursive`,
    `loadEntitiesRecursive`,
    `loadStaticEntity`, the spawner block
    entity's load/save): the logic is
    identical — passengers recurse and ride,
    non-compound list entries are skipped,
    the spawner keys are item 41's. The
    Rust was already compound-based
    throughout, so no code changes.
  - Six doc citations reworded from the
    1.21.6 `TagValueInput`/`ValueInput`
    names to 1.21.5's `CompoundTag` methods
    (`getBooleanOr`, `getFloatOr`,
    `getIntOr`, `read`): entity_nbt.rs (2),
    trial_spawner.rs, item_data.rs,
    persist/display.rs,
    brewing_speed_load_tests.rs. The
    block_entity_nbt.rs mention stays: it
    correctly contrasts 1.21.5's
    `optionalFieldOf` omission with 1.21.6's
    `ValueOutput.store` (item 35).
  - NOT this item: the `HAPPY_GHAST`
    `EntityType` removal belongs to item 58
    (happy-ghast removal), with the model
    layers, the mob, the render ropes and
    the leash branches.
  - Verification: `cargo check -p
    horizon-client -p horizon-client-core -p
    horizon-items --all-targets` 0 errors /
    0 warnings (1m45s).
- **Item 44 — Brushable / Crafter / Beehive /
  Skull / Furnace / Barrel / Brewing /
  Chest / CreakingHeart / SculkShrieker /
  TestBlock block entities, ported:**
  - Real fix, beehive: `IGNORED_BEE_TAGS`
    is 1.21.5's 29-key list — equipment is
    still `ArmorItems`/`HandItems` (the
    `equipment` component key is 1.21.6's)
    and the sleeping position is
    `SleepingX`/`SleepingY`/`SleepingZ`
    (not `sleeping_pos`). The rewrite also
    restores `HurtByTimestamp`, missing
    from the Rust's list against both
    versions.
  - The rest is the ValueInput/ValueOutput
    → CompoundTag migration with identical
    keys and defaults (loot tables,
    container items, furnace timers and
    `RecipesUsed`, `BrewTime`/`Fuel`,
    `creaking`, `warning_level`/`listener`,
    test-block mode/message/powered, skull
    profile/note_block_sound/custom_name);
    the Rust was already compound-based.
    `removeComponentsFromTag` (`discard` →
    `remove`) already strips the same keys
    in block_entity_item.rs. The furnace's
    `level()` → `serverLevel()` is the same
    level for a ServerPlayer; the
    test-block visibility change has no Rust
    surface.
  - Verification: `cargo check -p
    horizon-client --all-targets` 0 errors /
    0 warnings (1m15s).
- **Item 45 — ContainerHelper / LockCode /
  RandomizableContainer, ported:**
  - `ItemStackWithSlot` (1.21.6-only) is
    gone from the tree; the container
    codec is 1.21.5's manual list form
    (`Slot` byte + `ItemStack.save`), which
    the Rust's `encode_items`/`decode_items`
    already matched — including the
    missing-`Slot`-defaults-to-0 rule
    (`getByteOr("Slot", 0) & 255`) and
    unparseable-entries-become-EMPTY.
  - Real fix: the canonicalizer's `items()`
    dropped Slot-less entries instead of
    defaulting them to slot 0; it now
    defaults like the game path and
    1.21.5's `loadAllItems`.
  - No-ops verified: `LockCode` (`lock`
    key, `NO_LOCK` default) and the loot
    table/seed handling are unchanged
    apart from the CompoundTag signatures;
    the save-empty rule (the 3-arg
    `saveAllItems` always writes `Items`)
    matches the Rust's unconditional
    writes.
  - Verification: `cargo check -p
    horizon-client-core -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (1m20s).
- **Item 46 — ModelPart / BannerRenderer /
  ShieldSpecialRenderer /
  SpecialModelRenderer / PlayerHeadSpecialRenderer,
  ported:**
  - `SpecialModelRenderer.getExtents` is
    1.21.6-only: in 1.21.5 every special
    model gets the unit cube's corners as
    its extents (`SpecialModelWrapper.EXTENTS`),
    so `special_extents` returns the eight
    corners and the per-model computation
    (`special::extents`, `extents_of`, the
    `ModelPart.getExtentsForGui` port) is
    deleted. `bounding_box`/`posed_bounds`
    stay — 1.21.5 has `visitExtents` /
    `getModelBoundingBox` too, and the
    dropped-item placement, cluster poses
    and shelf alignment all read it.
  - `PlayerHeadSpecialRenderer` is
    1.21.6-only: in 1.21.5 a player head
    is `SkullSpecialRenderer` with kind
    `player`. `SpecialKind::PlayerHead`
    and the `minecraft:player_head` codec
    id are removed; a `Head` with kind
    `player` takes the profile argument
    and draws the profile's skin
    translucently, as the old arm did.
    The worn-head sites (armor stands,
    mobs) now build `Head` for every kind.
  - The `createPartLookup` citations in
    anim.rs are reworded to the Rust's own
    first-wins rule (1.21.5 has no such
    method). `getAllParts` (List vs Stream)
    has no Rust surface; the traversal
    order is the same either way.
  - NOT this item: `oversized_in_gui`
    belongs to items 101
    (`ItemStackRenderState`) and 121
    (`ItemModelResolver`); the registry
    change (`SpecialModelRenderers`) is
    item 208.
  - Verification: `cargo check -p
    horizon-assets -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (2m08s).
- **Item 47 — ToastManager, ported:**
  - The Now Playing toast is already gone
    (item 37): no `nowPlayingToast` field,
    no show/hide/create/remove, no Options
    in the constructor. The slide (600 ms,
    squared easing, `fullyVisibleFor`),
    the five 32-pixel slots with a queue,
    and the render formula (`guiWidth -
    width * visiblePortion`, `slot * 32`)
    already match 1.21.5's
    `ToastInstance.render` — 1.21.6's
    `xPos`/`yPos` methods, `nextStratum`
    and `onFinishedRendering` have no Rust
    surface (finished toasts are removed
    in update, so the missing render guard
    is unobservable).
  - Real fix, found while verifying: a
    system toast's width was
    `max(160, widest) + 30` (minimum 190);
    vanilla's is `max(160, 30 + widest)`
    (minimum 160). Fixed, with the
    `xPos` citation reworded to
    `ToastInstance.render`. The `multiline`
    form (message split at 200px) is not
    modelled — a pre-existing gap, same in
    1.21.6.
  - Verification: `cargo check -p
    horizon-client --all-targets` 0 errors /
    0 warnings (1m20s).
- **Item 48 — StructureTemplate /
  BlockEntityWithBoundingBoxRenderer,
  ported:**
  - Real fix: 1.21.5's `fillFromWorld`
    takes a single block to skip
    (structure void, always); the
    `ignoreBlocks` list is 1.21.6's, and
    only test instances used it (for air).
    The `ignore_air` parameter is removed
    from `save_structure_area` /
    `template_from_world`: test-instance
    saves now include air blocks, as
    1.21.5's `TestInstanceBlockEntity`
    does.
  - The rest is the ProblemReporter /
    TagValueInput / TagValueOutput →
    CompoundTag migration with identical
    logic (`fillEntityList`, `place` with
    its barrier pre-place, loot-seed
    reroll, waterlogging and shape
    update, `placeEntities` with its Pos
    rewrite and UUID removal). The
    `shouldRenderOffScreen` change is a
    signature only (both return true);
    the Rust's name-based never-cull set
    is unchanged.
  - Verification: `cargo check -p
    horizon-client --all-targets` 0 errors /
    0 warnings (1m10s).
- **Item 49 — CommandBlockEntity /
  BaseCommandBlock, verified, no code
  change:**
  - The diff is the ValueInput/ValueOutput
    → CompoundTag migration with identical
    keys, defaults and order (Command,
    SuccessCount, CustomName, TrackOutput
    default true, LastOutput only when
    tracking, UpdateLastExecution default
    true, LastExecution only when updating
    and set, powered, conditionMet, auto);
    the Rust's `CommandBlock::load`/`save`
    was already compound-based and matches
    on every point, as does the
    `removeComponentsFromTag` modelling
    (CustomName/conditionMet/powered
    stripped). The `save` return value and
    the `level()` →
    `getCommandSenderWorld()` client check
    have no Rust surface (both are the
    player's world).
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (25s).
- **Item 50 — Options, ported:**
  - `key.quickActions` is 1.21.6-only: the
    binding, the `QuickActions` action, the
    gamepad entries and the G handler are
    removed, and the whole
    `game/quick_actions.rs` module (the
    dialog-tag lookup) is deleted. The
    `minecraft:quick_actions` dialog
    registry entries in `lan/data.rs` and
    the command corpus stay for the dialog
    item.
  - `cloudRange` is 1.21.6-only: the
    setting, its options-file key, the
    `Opt::CloudRange` slider, the preset
    values and the fog cap are removed.
    1.21.5's cloud mesh is a fixed ±32-cell
    square, so `radius_cells` becomes the
    `CLOUD_RADIUS_CELLS = 32` constant,
    the fog cloud end is the dimension's
    `visual/cloud_fog_end_distance`
    uncapped, and the capture record and
    the parity fixture follow. The options
    parity key list and the perf-bench
    fixture drop `cloudRange` too.
  - `getCloudsType` (1.21.5) forces clouds
    OFF below an effective render distance
    of 4: `clouds::visible` takes the
    render distance and gates on it (the
    integrated server's view distance
    follows the render distance, so that
    is the effective one).
  - `final_source_volume` keeps its
    computation but is re-cited: 1.21.5
    multiplies the source volume in
    `calculateVolume` and applies master
    as the listener gain, which is the
    same product the fold computes;
    `getFinalSoundSourceVolume` is the
    1.21.6 name for the fold.
  - No-ops: `musicFrequency` /
    `showNowPlayingToast` (item 37), the
    glint-strength callback (the
    `glintStrength` value is read from the
    options file once per frame by
    `capture_glint_frame` and fed to the
    shaders as `GlintFrame.alpha`, so it
    is always current at draw time, as
    1.21.5's `setShaderGlintAlpha` call
    makes it — see item 54), the
    options-file JSON
    leniency (Gson's `JsonParser` and
    `LenientJsonParser` are both lenient),
    the `save()` version line (same value
    via the pin).
  - Known gap: the cloud-render-parity
    oracle still targets the 26.3
    `CloudRenderer` signatures
    (`buildMesh` with a `ByteBuffer` and a
    radius) and cannot run against
    1.21.5's `BufferBuilder`-based
    builder; re-porting it is harness work
    beyond this item.
  - Verification: `cargo check -p
    horizon-client-input -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (2m19s).
- **Item 51 — GameNarrator, ported:**
  - The 1.21.6 names go back to 1.21.5's:
    `say_chat` (chat gate, and — unlike
    the others — no empty check, so an
    empty message is still logged and
    said), `say` (system gate, empty
    messages skipped), `say_now` (system
    gate, interrupts). `saySystemChatQueued`
    has no 1.21.5 equivalent and is
    deleted with its `should_narrate_system_or_chat`
    gate.
  - Real behaviour changes at its two call
    sites: `handleSystemMessage` says
    system chat with the system gate only
    (1.21.6 narrated it at Chat status
    too), and `handleChatMessageError`
    does not narrate at all in 1.21.5, so
    the error branch's narration is
    removed. The volume citations are
    reworded to `narrateMessage`'s inline
    voice-times-master product (the
    computation was already that product).
  - The game-test `ReportGameListener.say`
    helper is renamed `say_report`: `Game::say`
    is now the narrator's.
  - Verification: `cargo check -p
    horizon-narrator -p horizon-client-render
    -p horizon-client --all-targets`
    0 errors / 0 warnings (2m33s).
- **Item 52 — Commands, verified, no code
  change:**
  - The `/dialog`, `/version` and
    `/waypoint` registrations are gone
    already (item 36), with no remnants in
    the tree, the order list or the
    dispatch; the restricted-flag
    `COMMAND_NODE_INSPECTOR` went with
    them. The LAN tree builder already
    matches 1.21.5's `fillUsableCommands`:
    it filters by `canUse` (required level
    against the player's), copies
    `executable`, and carries no
    requirements on the synced nodes — the
    1.21.5 `requires(() -> true)` form —
    with suggestion providers mapped from
    the report. `COMMAND_PREFIX` /
    `trimOptionalPrefix`, the
    `hasPermission` / `PermissionCheck`
    helper and the two `register`
    signature differences are Java API
    with no Rust surface.
  - Verification: `cargo check -p
    horizon-command -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (9s).
- **Item 53 — Vex, ported:**
  - The owner is a plain field in 1.21.5,
    never saved: the `EntityReference`
    model is removed — `owner()` reads
    `counter2` directly with no UUID
    lookup, `write_nbt` writes no `owner`
    key (and drops a stale one), and
    `read_nbt` does not read it. A
    summoned vex loses its evoker across
    a chunk save and copies no target
    after reloading; the tests assert the
    loss and the stale-key drop. The
    `CopyOwnerTargetGoal.start` null
    check is gone in 1.21.5 too, but
    `can_use` gates on a non-null owner
    with a target, so the lookup in
    `start` finds one whenever it runs.
    The bound origin and limited life are
    unchanged.
  - Verification: `cargo check -p
    horizon-client --all-targets` 0 errors /
    0 warnings (1m28s).
- **Item 54 — RenderSystem, verified, no
  code change:**
  - 1.21.6 moved the shader state (fog,
    lights, projection, glint alpha, game
    time, shader colour) into GPU buffer
    slices and `DynamicUniforms`;
    1.21.5 keeps them as plain CPU-side
    fields. The Rust is value-based
    throughout and already matches the
    1.21.5 form: the shader game time is
    computed with 1.21.5's exact formula
    (`(game_time % 24000 + partial) /
    24000`), the GUI diffuse lighting
    uses 1.21.5's exact matrices (flat:
    `rotationY(-PI/8).rotateX(3PI/4)`; 3D:
    the scale and `rotateYXZ(1.0821041,
    3.2375858, 0)` chain), fog and
    projection are plain values, and
    there is no `DynamicUniforms` /
    global-settings-uniform /
    `bindDefaultUniforms` modelling to
    remove. The glint alpha is read from
    the options file once per frame
    (`capture_glint_frame`), so it is
    always current at draw time, as
    1.21.5's `setShaderGlintAlpha`
    callback makes it — this corrects
    item 50's note, which wrongly said
    the value was never consumed.
    `setShaderColor` and the scissor
    state have no Rust surface beyond
    per-draw colours and the GUI scissor
    rect.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (7s).
- **Item 55 — Gui, ported:**
  - The contextual bar system is
    1.21.6-only and is gone:
    `ContextualBar` (the enum and its
    `next` state machine, with the
    `contextual_bar_priorities` test)
    is deleted from `locator_bar.rs`,
    `mount_jumping`
    (`willPrioritizeJumpInfo`) is
    deleted from `mounts.rs`, and the
    HUD layer is the 1.21.5 direct
    form: the jump bar when riding a
    jumpable vehicle, else the
    experience bar — no waypoint /
    recency / jump prioritisation.
    The layer id becomes
    `minecraft:experience_bar` (the
    1.21.6 `minecraft:contextual_info_bar`
    id no longer exists) and the
    native-hooks layer list follows.
    The Game-side locator-bar draw
    path goes with it:
    `locator_bar_draws`,
    `waypoint_entity`, the
    `locator_camera` field (and its
    per-frame set), and
    `Remote::player_entity`, whose
    only caller was the waypoint
    lookup. The lib `locator_bar`
    module (waypoint state, styles,
    `background`, `draw`) stays for
    item 108; the LAN waypoint sync
    stays for item 86.
  - The experience level is gated on
    the bar's visibility:
    1.21.5's `renderExperienceLevel`
    draws only when
    `isExperienceBarVisible` (no
    jumpable vehicle, has
    experience), so a rider sees
    neither the bar nor the level —
    the Rust drew the level while
    riding. The bar formula
    (`(progress * 183) as i32`,
    `h - 32 + 3`) and the level
    position (`h - 31 - 4`, centred)
    already matched.
  - The jump bar fill is
    `(scale * 183.0) as i32`:
    1.21.5's `renderJumpMeter` uses
    the 183-wide fill like the XP
    bar; the Rust used
    `Mth.lerpDiscrete` (1.21.6's
    `JumpableVehicleBar` form,
    topping out at 182). Position,
    background and cooldown sprite
    already matched.
  - Effects icons are deferred:
    1.21.5 draws every background
    first, then runs the icon blits
    (`MobEffectTextureManager` lookups
    resolve the same `mob_effect/<id>`
    sprites the Rust builds). The
    Rust drew each icon right after
    its background; the cells never
    overlap, so this is order-only.
  - The overlay message, the title
    (with its subtitle) and the
    saving indicator draw only above
    alpha 8 in 1.21.5; the Rust
    gated on `> 0` (the 1.21.6 form).
    The selected-item name keeps its
    `> 0` gate — both versions use
    it there.
  - No-ops: the debug crosshair gate
    (1.21.5's inline form is inside
    the first-person branch, as is
    1.21.6's extracted
    `shouldRenderDebugCrosshair`;
    the Rust's `debug_crosshair_drawn`
    already requires first person),
    the crosshair and attack-indicator
    geometry (identical), the
    spectator tooltip (already
    `renderTooltip`, the 1.21.5 name),
    `setNowPlaying` (already `say_now`
    under item 51), the
    `ReceivingLevelScreen` gate
    (1.21.6-only, no Rust surface),
    the hotbar's `z = -90` translate
    and the spyglass fill `z` (draw
    order, handled by the batch),
    `RenderType` vs `RenderPipelines`,
    `pushPose` vs `pushMatrix`,
    `nextStratum`, and the
    `Gui$RenderFunction` interface
    (no Rust surface).
  - Known gap (pre-existing, not a
    delta): the Rust's layer order
    (effects and boss bars before the
    health rows; scoreboard after the
    title; subtitles before chat)
    matches neither version's
    `LayeredDraw` order; the regions
    never overlap, so nothing visible
    differs.
  - Verification: `cargo check -j 1 -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (5m36s; the first run OOM'd at default
    parallelism under 3-stream load and
    also surfaced the now-deleted
    `player_entity` dead code).
- **Item 56 — TrialSpawner / Jukebox /
  SculkCatalyst / SculkSensor / Vault
  block entities, verified, no code
  change:**
  - The diff is the ValueInput /
    ValueOutput → CompoundTag
    migration with identical keys,
    defaults and order. The Rust is
    already compound-based
    throughout and matches on every
    point: the trial spawner load
    REPLACES the whole spawner
    (`**s = TrialSpawner::load(tag)`,
    1.21.5's `tag.read(codec)
    .orElseGet(default)`; 1.21.6
    mutated in place), with the
    config defaulting from the
    registry when the tag lacks one;
    the vault reads `config` /
    `server_data` / `shared_data`
    with the config defaulting and
    `VaultServerData.set` leaving
    the ejection total out (so a
    vault loaded mid-ejection counts
    from 0), and saves all three
    keys; the jukebox writes
    `RecordItem` only when non-empty
    and `ticks_since_song_started`
    only when the song clock is
    present; the sensor writes
    `last_vibration_frequency`
    (default 0) and `listener`, the
    catalyst its `cursors`.
  - The sculk `load` keeps a field
    when its key is absent rather
    than resetting it, but the only
    caller is the chunk-load path,
    where the entity is freshly
    constructed — identical to
    1.21.5's `orElseGet(Data::new)`.
    The jukebox stop-on-change
    branch cannot fire on a fresh
    load either (the item starts
    empty), as in vanilla.
  - The update tag already cites
    1.21.5's `TrialSpawnerData
    .getUpdateTag` (via `getData()`,
    not 1.21.6's `getStateData()`),
    and the components codec is
    already cited as
    `COMPONENTS_CODEC.optionalFieldOf`
    (item 35). No `ValueInput` /
    `ValueOutput` citations remain
    in the tree. `problemPath` /
    `BlockEntityPathElement`,
    `getNameForReporting`'s
    visibility, `saveCustomAndMetadata`
    and the `loadAdditional` ctor
    shapes have no Rust surface.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check, 5s).
- **Item 57 — SectionRenderDispatcher /
  WorldBorderRenderer / FogRenderer$FogMode,
  verified, no code change:**
  - The section dispatcher diff is
    GPU architecture: 1.21.6's
    `CompiledSectionMesh` /
    `ChunkSectionLayer` upload
    machinery against 1.21.5's
    `CompiledSection` +
    per-`RenderType` `SectionBuffers`.
    The Rust's wgpu mesher has its
    own section meshes and no
    surface for any of it — no
    `CompiledSectionMesh`, no
    dispatcher queue or stats
    (`pC` / `pU` / `aB`), no
    `toBatchCount`, no per-section
    global block entity tracking
    (item 25).
  - The one behavioural delta,
    `hasAllNeighbors`, has no Rust
    surface either: 1.21.5
    short-circuits to true within
    24 blocks of the player
    (`getDistToPlayerSqr() <=
    576.0`), 1.21.6 always checks
    all eight neighbours — but the
    Rust never gates meshing on
    neighbours at all. It meshes
    with missing neighbours by
    continuing the edge column
    (`Source::Missing`), a
    deliberate divergence already
    documented at mesher.rs ("Vanilla
    draws no chunk without its
    neighbours").
  - The translucency resort matches
    both versions: the Rust's
    `SortState::update` computes
    the same per-axis clamped
    point of view and skips the
    resort on an equal POV for a
    diagonal view (1.21.5's
    `pov.equals(pointOfView)` is
    1.21.6's `!isDifferentPointOfView`;
    the Rust's same-block
    refinement for aligned views is
    documented there and identical
    in both versions).
  - The world border math is
    identical in both versions and
    matches the Rust: alpha
    `(1 - distance / renderDistance)^4`
    clamped, the UV scroll
    `(millis % 3000) / 3000`,
    `v0 = -frac(camera.y * 0.5)`,
    the status colour channels, and
    sides drawn only within the
    render distance. The diff is
    GPU API only (`setShaderColor` /
    `setTextureMatrix` /
    `setModelOffset` vs
    `DynamicUniforms`, `GpuTexture`
    vs `GpuTextureView`,
    `setFilter` vs `setUseMipmaps`).
  - `FogRenderer$FogMode` has no
    diff content in this item's
    file (a report false positive);
    FogRenderer itself was ported
    to the 1.21.5 form under item 28.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check, 4s).
- **Item 58 — Ghast / HappyGhast /
  TemptGoal$ForNonPathfinders, ported:**
  the happy ghast is removed wholesale
  (entity, ghastling, harness, ropes,
  `happy_ghast_one_cm`), and the plain
  ghast's flight is already the 1.21.5
  form.
  - `crates/horizon-client/src/mobs/happy_ghast.rs`
    (754 lines) deleted, with
    `Species::HappyGhast`,
    `Family::HappyGhast`,
    `Anim::HappyGhast`,
    `LookKind::HappyGhast`,
    `Kind::Ghastling` and the
    ghastling brain package,
    `Sensor::AdultAnyType` /
    `sense_adult` /
    `followable_friendly`, the
    harness and ropes layers, the
    `happy_ghast_food` tag, the
    baby scale, the still-timeout
    save/load and the leash
    dispatch. `game/ghasts.rs`
    renamed to `game/entity_collisions.rs`
    and trimmed to the three
    collision fns (`interact_happy_ghast`,
    `dismount_happy_ghast`,
    `spawn_ghastling` deleted).
  - The 1.21.5 ghast needs no code
    change: it extends `FlyingMob`
    with the fixed-0.1 move control,
    the stepped `canReach` and the
    single-pick float goal, which is
    what `mobs/nether.rs` already
    does (item 10). The 1.21.6
    additions — `getSuitableFlyToPosition`
    (64 tries + heightmap mirror),
    the `FLYING_SPEED * 5 / 3`
    acceleration, `CAMERA_DISTANCE`
    (8.0) / `FLYING_SPEED` (0.06)
    attributes, the quad-leash
    overrides — are all gone with
    the happy ghast. `TemptGoal`
    is `PathfinderMob`-only again
    with the fixed 6.25 stop
    distance; `ForNonPathfinders`
    (move-control navigation to a
    random fraction toward the
    player's eye) was 1.21.6-only.
  - Collateral deletions:
    `Seen.still` (no readers left),
    `animal_brain::grown_up` (no
    callers left — no 1.21.5
    animal's brain stops on
    growing up) and `B::reset`
    (its only caller was
    `grown_up`), the `CullGrow`
    `height` parameter and the
    down-growth (no species grows
    down now), the `happy_ghast_one_cm`
    statistic (scoreboard list and
    stats screen), the parrot's
    silent happy-ghast imitation
    (`IMITATIONS` 41 → 40), the
    `happy_ghast` shadow radius and
    CEM tentacle mapping, the
    `minecraft:happy_ghast`
    anticheat carrier, the
    still-timeout NBT round-trip
    test, the harnessed-ghast
    client-moves test, and the
    `white_harness` dispenser case.
  - Tools: `tools/test_happy_ghast_leash_parity.py`
    deleted (it drove the deleted
    Rust test); the mob-AI parity
    scenarios, the LAN e2e mount
    list (and the dead `rise`
    branch), the entity-model and
    dispenser and death-loot and
    finalize-spawn lists all drop
    the happy ghast. The three
    leash/passenger corpora keep a
    20-health holder by summoning a
    husk instead (a zombie would
    burn in daylight). The port map
    loses 33 rows for classes and
    methods absent in 1.21.5
    (`HappyGhast*`,
    `HappyGhastRenderer`,
    `HappyGhastModel`,
    `HappyGhastHarnessModel`,
    `TemptGoal.ForNonPathfinders`,
    `RopesLayer`,
    `AdultSensorAnyType`, the
    `RandomFloatAroundGoal` helpers)
    and the surviving rows' notes
    drop the ghast clauses.
  - The command-exec oracle
    (`VanillaCommandExecOracle.writeHappyGhastLeashReference`)
    referenced
    `net.minecraft.world.entity.animal.happyghast.HappyGhast`
    directly, which does not exist
    in 1.21.5 — the oracle could not
    compile against the 1.21.5
    server jar. The method and its
    call site are deleted.
  - Not this item: the dried ghast
    block and its sounds
    (`animate_tick.rs`,
    `timeline.rs::dried_ghast_tick`,
    the orphaned `spawn_ghastling`
    event) belong to item 182
    (`DriedGhastBlock`, removed);
    the `spawn_ghastling` event is
    currently unhandled (the
    Game-side arm is gone).
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (3m38s full, 49s re-check after
    removing the dead `B::reset`).
- **Item 59 — GUI components
  (MultiLineEditBox, DebugScreenOverlay,
  MultiLineTextWidget,
  FocusableTextWidget, ChatComponent,
  AbstractSelectionList,
  BossHealthOverlay, CycleButton,
  AbstractButton, AbstractScrollArea,
  AbstractSliderButton,
  FittingMultiLineTextWidget,
  ImageButton, StringWidget), ported:**
  - The multiline edit box loses its
    line limit: `setLineLimit` /
    `overflowsLineLimit` are
    1.21.6-only, so `line_limit`,
    `set_line_limit` and the limit
    checks in `set_value` /
    `insert_text` go, with the
    line-limit test and the dialog
    field's `set_line_limit` call
    (the dialog still sizes the box
    from `max_lines`). The rest
    already matches 1.21.5: fixed
    text colour 0xE0E0E0, cursor
    0xD0D0D0, shadowed text, the
    character-limit label, and the
    cursor-at-line-end rule
    (`cursor <= end` renders inline;
    1.21.6's `<` falls to the "_"
    append). The configurable
    colours/shadow and the Builder
    never had a Rust surface.
  - The debug 3D crosshair is a
    different drawing in 1.21.5:
    screen-space at the screen
    centre, 10 units long, rotated
    by the camera's pitch and yaw
    (`Gui.renderCrosshair`'s model
    view), not the 1.21.6 world-space
    gizmo 1 block ahead of the
    camera at 0.01 * gui scale.
    `hud_layout::debug_crosshair_3d`
    computes the six quads (black
    4-device-pixel arms under the
    coloured 2-device-pixel ones;
    an arm along the view direction
    has no screen length and is
    skipped), drawn in the
    `minecraft:crosshair` layer
    under the same gate as the 2D
    sprite, with the main camera's
    rotation (`posed_rotation` or
    the player's yaw/pitch). The
    world-space path is deleted:
    `Gizmos::debug_crosshair`, the
    game.rs producer, the
    `debug_crosshair_drawn` gate,
    the `Frame.debug_crosshair`
    field, both gfx pipelines, the
    crosshair globals/group and
    the render pass, and the field
    inits in the examples, capture
    and pack tests.
  - Chat: 1.21.5 draws a line only
    while its text alpha is over 3
    — at 3 or below it draws no
    background, tag or text, though
    it still counts for the
    scrollbar. The Rust had the
    1.21.6 `forEachLine` form
    (background + tag for every
    line over the 1e-5 fade floor);
    both passes now skip lines at
    text alpha 3 or below. The
    fade gate itself agrees at
    integer ticks (age 199 kept,
    age 200 dropped, in both
    forms), and the count is
    unchanged. The tag icon was
    never drawn — a pre-existing
    gap, the same in both versions.
  - No Rust surface: the
    MultiLineTextWidget hover/click
    handling (`configureStyleHandling`,
    `getComponentStyleAt` — the
    Rust's text widget only
    narrates), the
    FocusableTextWidget
    `fillBackground` (only dialog
    code references it, and the
    dialog item owns that), the
    CycleButton
    `displayOnlyValue(boolean)`
    overload (only the no-arg form
    is named, in a narration
    comment), the slider
    `DEFAULT_HEIGHT`, and the
    ScrollableLayout class (the
    screens' own scroll areas stay;
    the dialog item owns the dialog
    one). StringWidget has no diff
    content (a report false
    positive — its one method is
    the inherited `getColor`,
    item 350). The rest is GPU-API
    churn (`RenderPipelines` →
    `RenderType`, `pushMatrix` →
    `pushPose`, `nextStratum`) or
    same-value colour math (`-1`
    vs `16777215`,
    `ARGB.color(alpha, …)` vs
    `… | ceil(alpha*255)<<24`).
  - The port map drops the
    `ChatComponent.forEachLine`
    row (replaced by a
    `ChatComponent.render` row),
    the `MultiLineEditBox.Builder.build`
    row and all 11
    `ScrollableLayout` rows, and
    gains a
    `DebugScreenOverlay.render3dCrosshair`
    row. The
    `MultilineTextField.setLineLimit`
    row stays for item 307 (rank 3).
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (1m46s).
- **Item 60 — screens and widgets
  (AccessibilityOnboardingScreen,
  Toast, LogoRenderer,
  WidgetTooltipHolder,
  PresetFlatWorldScreen,
  BackupConfirmScreen,
  GenericWaitingScreen,
  OptimizeWorldScreen,
  SelectWorldScreen), ported:**
  - The accessibility onboarding
    screen loses its fades: the
    two-second fade in and
    one-second fade out are
    1.21.6-only. In 1.21.5 the
    screen appears at once and
    `onClose` closes it at once
    (`close(true, onClose)`), so
    the `Onboarding` fade state
    (`fade_in_start`, `fading_in`,
    `fade_out_start`, `alpha`),
    the fade math, `poll_onboarding`
    and the per-frame poll go;
    `onboarding_close` now switches
    to the title and marks
    onboarding finished directly,
    and the logo draws at full
    alpha after the buttons. The
    fade test is replaced by one
    asserting Escape closes at
    once. The panorama call in
    `renderPanorama` differs too,
    but that belongs to the
    panorama item (292).
  - Flat-world preset strings keep
    their zero-height layers: the
    per-layer clamp to what fits
    under Y_SIZE exists in both
    versions (`getLayerInfoFromString`),
    but only 1.21.6 drops layers
    once the total reaches Y_SIZE
    (`heightLimited` + the
    `remaining > 0` guard in
    `getLayersInfoFromString`).
    `FlatSettings::parse` no longer
    skips zero-height layers.
  - No Rust surface: the `Toast`
    `xPos` / `yPos` /
    `onFinishedRendering` defaults
    (the Rust computes the slide
    position inline, the same
    formula in both versions),
    `LogoRenderer.keepLogoThroughFade`
    (the getter only; the field
    still exists in 1.21.5), the
    `WidgetTooltipHolder`
    signature change (the delay
    logic is identical; only the
    tooltip positioning is
    modelled), and the
    PresetFlatWorldScreen z=400
    title lift (the Rust draws
    screens in its own order).
    The four inner-class entries
    (`CreateFlatWorldScreen$DetailsList$Entry`,
    `PresetFlatWorldScreen$PresetsList$Entry`,
    `PlainTextButton`,
    `ConfirmExperimentalFeaturesScreen$DetailsScreen$PackListEntry`)
    have no diff content at all —
    report false positives. The
    rest is same-value colour math
    (`-1` vs `16777215`,
    `-6250336` vs `10526880`) and
    `RenderType` churn.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (1m25s).
- **Item 61 — VariantUtils /
  PatrollingMonster / ProtoChunk,
  verified, citations only:**
  - `VariantUtils.selectVariantToSpawn`
    is 1.21.6-only: 1.21.5 has the
    same selection as per-class
    static methods
    (`PigVariants` / `CowVariants` /
    `ChickenVariants` / `CatVariants` /
    `FrogVariants.selectVariantToSpawn`,
    taking the random, the registry
    lookup and the `SpawnContext`).
    Both call `PriorityProvider.pick`
    over the registry's elements —
    `PriorityProvider` and
    `SpawnContext` are unchanged
    between the versions (neither
    appears in the report), and the
    variant data model
    (`spawn_conditions` with
    `priority`) is the same. The
    Rust's `VariantRegistry::select`
    (highest priority whose
    condition holds, one at random)
    is correct for both versions
    and is proven against the
    1.21.5 oracle by the
    finalize-spawn parity harness
    (pig, cow, chicken, wolf, cat,
    frog across the discriminating
    biomes). Only the two doc
    citations in finalize.rs
    changed, to `PriorityProvider.pick`
    and the per-class methods.
    (`CatVariants` / `WolfVariants`
    are item 69.)
  - The rest is the ValueInput /
    ValueOutput → CompoundTag
    migration with identical keys
    and defaults: `VariantUtils`
    read/write on the `variant`
    key, `PatrollingMonster` on
    `patrol_target` (nullable),
    `PatrolLeader` and `Patrolling`
    (both default false). The Rust
    is already compound-based and
    matches exactly
    (`raider::write_nbt` /
    `read_nbt`: the IntArray target
    removed when absent, the two
    flags as bytes). The
    `protected` → `public`
    visibility change on the
    save/load methods has no Rust
    surface.
  - `ProtoChunk.addEntity`: 1.21.6
    wraps the entity save in a
    `ProblemReporter.ScopedCollector`
    + `TagValueOutput`; 1.21.5
    saves straight to a
    `CompoundTag`. The reporter
    only collects problems for
    logging — the saved tag is the
    same, and the Rust has no
    problem reporter.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (55s).
- **Item 62 — ProblemReporter /
  ExtraCodecs / Mth, verified, no
  code change:**
  - `ProblemReporter` is a different
    interface in each version: 1.21.6
    has `forChild(PathElement)` /
    `report(Problem)` with the
    `PathElement` records, the tree
    report and the
    `ScopedCollector`; 1.21.5 has
    `forChild(String)` /
    `report(String)` with a
    `Multimap<String, String>` and
    `get()` / `getReport()` returning
    `Optional<String>`. The Rust has
    no problem reporter at all
    (zero references) — Horizon's
    loaders report errors through
    their own results — so neither
    form has a surface.
  - `ExtraCodecs.NBT`, `VECTOR2F`
    and `VECTOR3I` are 1.21.6-only
    additions. The Rust cites other
    `ExtraCodecs` (`RGB_COLOR_CODEC`,
    `MATRIX4F`, `UNTRUSTED_URI`,
    `ARGB_COLOR_CODEC`, `intRange`),
    all present in 1.21.5, and has
    no vector codec; its `Codec<Tag>`
    is the wire NBT codec, not the
    DFU `NBT` converter.
  - `Mth.smallestSquareSide`
    (`ceil(sqrt(n))`) is 1.21.6-only.
    The Rust's only `sqrt().ceil()`
    is the shader-pack block atlas
    grid in `PackAtlas::new`, an
    independent computation (vanilla's
    caller is the GUI item atlas,
    which has no Rust counterpart).
  - The port map drops 6 rows for
    methods absent in 1.21.5
    (`Mth.smallestSquareSide`,
    `Collector.hasFatalProblems`,
    `Collector.forEach`,
    `Collector.getTreeReport`,
    `ProblemTreeNode.getLines`,
    `ScopedCollector.close`);
    `Collector.getReport` stays —
    it exists in both versions
    (returning `Optional<String>`
    in 1.21.5, `String` in 1.21.6).
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check, 3s).
- **Item 63 — LeashFenceKnotEntity,
  verified, no code change**
  (already the 1.21.5 form from
  item 19):
  - `use_knot` is the 1.21.5
    `interact` exactly: the
    player's leashed mobs in the
    seven-block cube tie to the
    knot with no attach check
    (`bindPlayerMobs`); with
    nothing to tie, the knot
    discards itself silently and
    a creative player takes the
    leads hanging on it off
    without dropping them; the
    BLOCK_ATTACH game event fires
    on an attach or a removal.
    No shears branch, no
    knot-to-player transfer, no
    `canHaveALeashAttachedTo`
    snap-range check, no sound
    on interact — all 1.21.6-only.
  - The sounds are the 1.21.5
    names: `entity.leash_knot.break`
    on a fence break
    (`tick_explicit_knots`),
    `entity.leash_knot.place`
    on the `LeadItem.useOn` tie
    only (the knot's own interact
    plays none). `LEAD_TIED` /
    `LEAD_UNTIED` are 1.21.6-only.
  - No `notifyLeasheeRemoved`: a
    knot outlives its last lead in
    1.21.5, so a re-tie reuses the
    same entity (`tie_knot`, and
    the knot-identity test says so
    outright). The save/load
    methods are empty in both
    versions.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check, 1s).
- **Item 64 — AnimalPanic /
  FollowTemptation / BabyFollowAdult /
  AxolotlAi, ported:**
  - `AnimalPanic` loses its flying
    form: the `positionGetter` /
    `AirAndWaterRandomPos` variant
    and the `(float, int)` ctor are
    1.21.6-only (for the ghastling),
    so the `fly` field goes and
    `panic_pos` is land-only
    (`LandRandomPos`, 5 out, 4 up
    or down). `start` no longer
    stops the navigation — 1.21.6
    added the `getNavigation().stop()`;
    1.21.5 only sets IS_PANICKING
    and erases the walk target.
  - `FollowTemptation` loses
    `lookInTheEyes` (the 3-arg ctor
    is 1.21.6-only): the field goes
    and the walk target always
    takes `EntityTracker(player,
    false)`. The IS_TEMPTED
    differences need no code: the
    Rust derives "tempted" from the
    tempting player's presence
    rather than modelling the
    memory, so 1.21.5's "set false
    on stop" vs 1.21.6's "erase on
    stop" and the start condition
    (REGISTERED vs VALUE_ABSENT)
    have no surface — the start
    check has no tempted gate,
    matching 1.21.5.
  - `BabyFollowAdult` is
    AgeableMob-only again: the
    `player` variant (following
    the nearest visible player)
    was the ghastling's, so the
    field goes and the trigger
    reads NEAREST_VISIBLE_ADULT
    only. The walk target takes
    `EntityTracker(adult, false)`
    and the look target
    `EntityTracker(adult, true)`,
    as in 1.21.5.
  - `EntityTracker`'s second
    boolean (`targetEyeHeight`,
    making `currentBlockPosition`
    the eye block) is 1.21.6-only:
    the `block_eye` field goes and
    `block()` always returns the
    feet block. The only
    `block_eye: true` sites were
    the FollowAdult trigger (dead
    in practice — only the axolotl
    package builds it) and the
    FollowTemptation tick (fed by
    the removed `look_in_the_eyes`).
  - `AdultSensorAnyType` and
    `HappyGhastAi` were already
    deleted under item 58. The
    `Stroll` `fly` field stays:
    `RandomStroll.fly` exists in
    1.21.5, no species just uses
    it now. The axolotl package's
    `BabyFollowAdult.create` call
    follows from the 2-arg form.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (58s).
- **Item 65 — SoundEngine /
  SimpleSoundInstance, ported:**
  - Zero volume stops sounds in
    1.21.5: `updateCategoryVolume`
    stops a channel whose calculated
    volume is zero instead of
    setting it, and the tick stops
    and removes sounds whose source
    volume is zero at once rather
    than after their delete time.
    `refresh_category_volume` now
    stops at option-volume zero
    (keyed on the option volumes,
    not the category gain — the
    gain is Horizon's fade/ducking,
    and a faded-out track must come
    back when the gain does, as
    vanilla fades music via
    per-instance `setVolume`), and
    the tick cleanup removes
    zero-source-volume sounds
    immediately without re-queueing
    them. The master-silence stop
    was already 1.21.5 (item 25).
  - `play` loses the MUSIC
    exception: 1.21.6 let music
    start silently at zero volume
    (and past a zero master gain);
    1.21.5 skips any zero-volume
    sound unless it can start
    silent. `StartedSilently`
    survives for the
    can-start-silent instances
    (jukebox, riding, bee, music
    layers).
  - Nothing ticks while paused:
    `tick_music_when_paused` (the
    1.21.6 cleanup of stopped
    music channels) is deleted.
    `pause()` already paused every
    channel (1.21.5's `pause()`,
    not 1.21.6's
    `pauseAllExcept(MUSIC, UI)`).
  - The UI source is gone: 1.21.5
    has 10 sound sources, no UI.
    `for_ui` plays on MASTER,
    `ui.*` events classify to
    MASTER, `SOUND_SOURCES` drops
    "ui", and every volume array
    (`SOURCE_VOLUMES`,
    `Settings.volumes`,
    `source_volumes`,
    `gain_by_source`, the sync
    volumes) shrinks from 11 to
    10. The options sliders and
    the options file follow from
    the source list, so they lose
    the UI slider with no further
    change. `for_music` was already
    fixed volume 1.0, and the Now
    Playing toast was already gone
    (item 37).
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (1m25s re-check after fixing a
    stale `UI` import in the sound
    preview test).
- **Item 66 — Equippable / Shulker,
  ported:**
  - The llama carpet shear goes:
    1.21.5 has no shear-equipment
    system (`attemptToShearEquipment`
    / `canBeSheared` / the shearing
    sound are 1.21.6-only — item 23
    established this and deleted
    the saddle path), and neither
    `Llama` nor `AbstractHorse`
    handles shears in 1.21.5, so
    shears on a carpeted llama
    fall through to the ride, as
    `AbstractHorse.mobInteract`
    does. (The wolf-armor shear
    stays: 1.21.5 has it inline in
    `Wolf.mobInteract`, item 21.)
    The `harness(DyeColor)`
    factory went with item 58.
  - The protocol layer needs no
    change: `Stack` always holds
    the pinned (current) component
    layout, and the `v1_21_5`
    adapter adds the default
    `canBeSheared` / shearing sound
    when reading 1.21.5 data up
    and strips them when writing
    down — so the hash and skip
    functions correctly read the
    five-bool form. The client
    `Equippable` structs (item
    sync, menus) carry only slot /
    asset / overlay / allowed
    entities.
  - Shulker: the save/load
    migration keeps identical keys
    and defaults (`AttachFace`
    DOWN, `Peek` 0, `Color` 16),
    and the Rust matches exactly.
    `canBeCollidedWith` drops its
    unused entity parameter in
    1.21.5; both forms return
    `isAlive()`, which is what
    `blocks_mob` does (its doc now
    cites the no-arg form).
  - The port map drops the two
    stale `Mob.attemptToShearEquipment`
    / `Mob.shearItem` rows, which
    still cited the `shear_saddle`
    item 23 deleted.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (47s re-check after dropping the
    now-unused `ItemStack` import).
- **Item 67 — CommandSuggestions /
  ChatScreen, verified:**
  - The generics change
    (`ClientSuggestionProvider` ->
    `SharedSuggestionProvider` on
    the dispatcher / parse /
    context types) is type-erased
    in Java and has no Rust
    equivalent: the Rust
    dispatcher and suggestion
    types carry no provider type
    parameter (only doc comments
    name the two providers).
  - The renames are API-only with
    identical behaviour:
    `setTooltipForNextFrame` ->
    `renderTooltip`, `saySystemNow`
    -> `sayNow`, `addRenderableWidget`
    -> `addWidget`,
    `RenderPipelines.GUI_TEXTURED`
    -> `RenderType::guiTextured`.
    The Rust narrator call is
    already `say_now`
    (`chat.rs:1794`).
  - The one real difference,
    `ChatScreen`'s render order,
    already matches 1.21.5: game.rs
    draws the chat lines
    (`draw_chat`) before the screen
    match calls `draw_chat_screen`,
    which fills the input
    background, draws the input,
    then the suggestions last —
    paint order is the 1.21.5
    z+200. (1.21.6 fills before
    the chat lines.)
  - The command-block label colours
    are the same values in a
    different literal form (-1 ==
    16777215, -6250336 == 10526880);
    the Rust block editor cites
    -6250336. No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 68 — AbstractArrow, ported:**
  - 1.21.5 clears a burning arrow's
    fire in powder snow too
    (`isInWaterOrRain() ||
    state.is(POWDER_SNOW)`); 1.21.6
    dropped the powder-snow arm.
    The arrow tick now checks the
    block at its position through
    a new `ArrowScene::is_powder_snow`
    (registry name, the same shape
    as motion's check).
  - The save/load change is Java
    API-only: 1.21.6 moved
    `addAdditionalSaveData` /
    `readAdditionalSaveData` to
    `ValueInput` / `ValueOutput`,
    1.21.5 keeps `CompoundTag` +
    a `RegistryOps` context, but
    the keys and values are
    identical (`life`,
    `inBlockState`, `shake`,
    `inGround`, `damage`, `pickup`,
    `crit`, `PierceLevel`,
    `SoundEvent`, `item`, `weapon`)
    and the Rust NBT code already
    writes and reads them all.
  - Test: `powder_snow_clears_a_burning_arrows_fire_like_water`
    mirrors the water test — a
    burning arrow flying through
    powder snow has `fire_ticks`
    0 after every tick, with
    `on_fire` following only the
    synced flag. The shared test
    registry gains powder snow
    (state 21).
  - Drive-by: item 55's removal
    of the contextual bar system
    left an orphaned doc comment
    and `#[derive]` above
    `locator_bar.rs`'s test module
    (E0774), which broke the whole
    `horizon-client-lib` test
    build; removed (the enum is
    gone, only the comment named
    it).
  - Verification: `cargo test -p
    horizon-client-lib --lib arrows::`
    8 passed, 1 ignored (the
    client-arrow parity fixture),
    and `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings.
- **Item 69 — TemptGoal / wolf & cat
  variants / sit goal, verified:**
  - `TemptGoal`: 1.21.5 hardcodes
    the stop distance (6.25 =
    2.5 squared); the Rust default
    is 2.5. The 1.21.6
    `ForNonPathfinders` subclass
    has no 1.21.5 user (it serves
    the happy ghast, already
    removed). The armadillo's
    1.0 / 2.0 stop distance maps
    the brain `FollowTemptation`
    close-enough distance, which
    is unchanged between the
    versions — not a `TemptGoal`
    parameter.
  - `ShoulderRidingEntity` and
    `Cat` save/load: Java API-only
    (`TagValueOutput` /
    `ValueInput` vs `CompoundTag` +
    `RegistryOps`). The content is
    identical (`id` + the entity
    tag; `CollarColor`, red
    default, which the Rust NBT
    code already reads and
    writes).
  - `WolfModel`: the right legs
    are unmirrored in 1.21.5, but
    the geometry comes from
    `entity-models.json`, exported
    from the version's own client
    jar at prepare — the 1.21.5
    data already has the 1.21.5
    form. No Rust model code
    names the legs' mirroring.
  - `SitWhenOrderedToGoal`: 1.21.6
    sits a pet whose owner is in
    another level; in 1.21.5 the
    owner lookup is level-scoped,
    so it returns null and sits
    anyway. The Rust lookup is
    level-scoped the same way
    (`w.player`), so a
    cross-dimension owner yields
    `None` and the pet sits.
  - Variant selection: 1.21.5's
    `CatVariants` /
    `WolfVariants.selectVariantToSpawn`
    and 1.21.6's `VariantUtils`
    form are the same
    `PriorityProvider.pick` over
    the same registry and
    selectors. The Rust
    `cat_variant` matches the
    moon cases (uniform over 11
    on a full moon, 10
    otherwise); the swamp-hut
    always-black case is a
    pre-existing gap in both
    versions, not a delta.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 70 — StringTag, verified:**
  - 1.21.6 adds
    `escapeWithoutQuotes` (and
    its `StringBuilder` overload);
    1.21.5 has no such method.
    The Rust port has it as the
    private `escape` in dialog.rs,
    used only by dialog template
    substitution (a text input's
    value going into a command).
    Nothing else in the tree
    cites it, and the rest of
    `StringTag` is unchanged
    between the versions.
  - Removing the function now
    would change dialog behaviour
    owned by the dialog item (the
    `dialog/` screens package and
    the `/dialog` command's
    leftovers are that item's;
    item 36 already took the
    command out of the tree), so
    it stays until the dialog
    item takes it out with the
    rest. No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 71 — Advancement triggers &
  toasts, ported:**
  - The trigger changes are Java
    typing and diagnostics only:
    `level()` -> `serverLevel()`
    narrows the type (same
    instance), the validation
    paths gain a leading dot
    (`"location"` -> `".location"`,
    and the entity fields), and
    `placedBlockWithProperties`
    is a 1.21.6-only datapack
    builder helper. The Rust port
    implements none of these
    surfaces (no criterion
    validator, no advancement
    builders). `AdvancementRewards.grant`
    is the same logic with the
    server reached through the
    player instead of the level.
  - `AdvancementToast` is the one
    real change, in the
    multi-line fade: 1.21.5 draws
    the frame label at full
    opacity (its `colour | alpha`
    is a no-op because the label
    colour already carries alpha
    0xFF), while 1.21.6 fades it
    out with `ARGB.color`; and
    the title fades in from alpha
    64 (1.21.5 ORs the fade with
    0x40), not from 0. The toast
    draw now matches, with the
    title alpha extracted as
    `advancement_title_alpha`.
  - `RecipeToast` and the `Toast`
    defaults are renames and
    1.21.6-only API (`xPos` /
    `yPos` / `onFinishedRendering`)
    with no Rust surface; the
    recipe toast's 0.6 item scale
    is unchanged.
  - Test:
    `advancement_title_fades_in_from_alpha_64`
    pins 1500 -> 64, 1550 -> 106,
    1800 -> 252.
  - Verification: `cargo test -p
    horizon-client --lib toast`
    10 passed, 1 ignored, plus
    `advancement_title_fades_in_from_alpha_64`,
    and `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings.
- **Item 72 — PistonMovingBlockEntity /
  ServerLevel / EntityStorage,
  verified:**
  - The piston's
    `removeLatestMovementRecording`
    -> `removeLatestMovementRecordingBatch`
    is a rename of a system the
    Rust port never implemented:
    no movement recording exists
    anywhere in the tree. The
    save/load change is Java
    API-only (`ValueInput` /
    `ValueOutput` vs `CompoundTag`
    + `RegistryOps`); the keys
    (`blockState`, `facing`,
    `progress`, `extending`) are
    identical and already pinned
    by
    `moving_piston_fields_match_vanilla`.
  - `ServerLevel`'s waypoint
    manager removal belongs to
    item 108 (the locator bar):
    the wire was unwired by the
    publish, the internal
    locator-bar / LAN tracker
    stays until that item.
    `updateNeighboursOnBlockSet`
    and `waitForChunkAndEntities`
    are 1.21.6-only methods with
    no Rust equivalent (the first
    is the standard neighbour-update
    sequence the Rust block-set
    code already does inline).
  - `EntityStorage` is Java
    API-only too: the stored form
    is the same (`Entities` list,
    `Position`, current
    `DataVersion`), and the Rust
    saver writes `Position` +
    `Entities` the same way.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 73 — Connection / server
  listeners, verified:**
  - The `protocolVersion()` /
    `name()` -> `getProtocolVersion()`
    / `getName()` changes are
    Java record-to-class accessor
    renames. The
    `ChannelFutureListener` ->
    `PacketSendListener` change
    is Netty send plumbing: the
    1.21.5 `doSendPacket` wraps
    the listener with an
    on-failure fallback packet,
    while 1.21.6 attaches the
    Netty listener directly.
    The Rust network layer has
    no per-packet send-listener
    concept at all, so neither
    form has a Rust surface.
  - `handleCustomClickAction`
    is already gone from the
    protocol (configuration.rs /
    play.rs carry the "1.21.5
    has no dialog packets"
    notes from the rewire).
  - The handshake version check
    already matches 1.21.5 in
    `lan/conn.rs`: protocol !=
    770 disconnects with
    `outdated_client` below 754
    and `incompatible` above,
    with the game version name
    as the argument.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 74 — MeleeAttackGoal /
  MoveToBlockGoal / RandomPos,
  verified:**
  - Pure renames: 1.21.6 renamed
    `isWithinRestriction` /
    `hasRestriction` /
    `getRestrictCenter` to
    `isWithinHome` / `hasHome` /
    `getHomePosition`; the
    checks are identical (a
    mob outside its restriction
    centre + radius fails the
    goal, and random positions
    bias back toward the
    centre). The Rust AI keeps
    the 1.21.6 `home` naming
    internally (`home: Option<(pos,
    radius)>`, radius -1 meaning
    unrestricted, matching
    vanilla), which is naming
    only — the behaviour is
    the same in both versions.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 75 — Dispenser / hopper
  block entities & screens,
  verified:**
  - The block-entity changes are
    Java API-only (`ValueInput` /
    `ValueOutput` vs `CompoundTag`
    + `Provider`); the stored
    form is identical (`Items`
    via `ContainerHelper`,
    `TransferCooldown` for the
    hopper). The Rust side
    already canonicalizes
    `TransferCooldown` (default
    -1) and reads it for the
    hopper's runtime cooldown
    in `machines.rs`.
  - The screen changes are a
    render-API rename
    (`RenderPipelines.GUI_TEXTURED`
    vs `RenderType::guiTextured`)
    with identical blit
    parameters (same texture,
    same centred position and
    256x256 sheet); the Rust
    GUI has its own pipeline.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 76 — `/playsound` to the
  1.21.5 form, other commands
  verified:**
  - `/playsound` is the one real
    change, in two places.
    1.21.6 only plays the sound
    to targets in the source's
    level; 1.21.5 has no level
    check, so every target
    hears it. The guest
    dimension filter is gone.
    And 1.21.6's feedback counts
    the players who heard it
    (and names the first
    recipient), while 1.21.5's
    counts the targets (and
    names the first target);
    the result still counts the
    hearers, and zero hearers
    still errors with
    `commands.playsound.failed`.
    `cmd_playsound` now counts
    hearers and targets
    separately instead of
    tracking recipients.
    The placement math
    (`playsound_placement`) is
    identical in both versions.
  - The rest is Java-only:
    `Commands.hasPermission(2)`
    vs the same lambda in
    Attribute / Damage /
    Particle / Ride / Rotate /
    SpreadPlayers / StopSound,
    `SuggestionProviders.cast`
    (generics) in PlaySound /
    StopSound, and
    `ResourceKeyArgument`'s
    suggestion path (static
    helper vs the same call on
    the source).
  - Verification: `cargo test -p
    horizon-client --lib
    world_cmds::tests` 5 passed,
    and `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings.
- **Item 77 — Display /
  MinecartFurnace / ArmorStand,
  verified:**
  - Display and MinecartFurnace
    are Java API-only
    (`ValueInput` / `ValueOutput`
    vs `CompoundTag`): the keys
    and defaults are identical
    (`transformation`,
    `interpolation_duration`,
    `start_interpolation`,
    `teleport_duration` clamped
    0..59, `billboard`,
    `view_range`, shadow and
    size fields, `block_state`,
    `item` / `item_display`,
    the text-display flags;
    `PushX` / `PushZ`). The Rust
    side already handles them
    (`persist.rs` display tests,
    `minecarts.rs` PushX/PushZ
    round-trip test). The
    text-display parse differs
    only in plumbing (codec
    parse vs `ValueInput.read`);
    the outcomes match.
  - ArmorStand is the one
    structural change, and the
    Rust port already matches
    the 1.21.5 form: 1.21.5
    saves only the non-default
    `Pose` parts and defaults
    missing parts on load,
    while 1.21.6 writes the
    whole `ArmorStandPose`
    record. `armor_stand_to_nbt`
    filters to non-default
    parts and
    `armor_stand_from_nbt`
    defaults the rest. The
    1.21.5 field-plus-tick-sync
    vs 1.21.6 entity-data-only
    split collapses in the
    Rust port's single `pose`
    field, which NBT load, LAN
    metadata (entries 16-21)
    and rendering all read.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 78 — Camel leash & model,
  verified:**
  - The leash system is already
    the 1.21.5 form (ported by
    the earlier leash item):
    `leash.rs` keeps the legacy
    pull past 6 blocks, the
    snap at 10 (1.21.6 raised
    it to 12), and no break
    sound. The camel's
    `handleLeashAtDistance` is
    implemented exactly as
    1.21.5 has it — distance >
    6 while sitting, not in
    transition, and able to
    change pose -> stand up —
    and there is no quad-leash
    offset or elastic-pull
    handler anywhere in the
    tree (only a doc note that
    1.21.6 replaced the legacy
    pull with the dampened-spring
    `Wrench` system).
  - Save/load is Java API-only
    (`LastPoseTick`, same key);
    the model change is the
    1.21.6 baked
    `KeyframeAnimation` vs the
    1.21.5 `animate` /
    `animateWalk` calls over
    the same `CamelAnimation`
    definitions with the same
    parameters — rendering API
    only. No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 79 — Raider / Pillager,
  verified:**
  - Pure save/load API changes
    (`ValueInput` / `ValueOutput`
    vs `CompoundTag` +
    `registryAccess`): the keys
    are identical (`Wave`,
    `CanJoinRaid`, `RaidId`,
    the pillager's `Inventory`
    via the shared carrier
    helpers). The Rust side
    already round-trips them
    (`mobs/raider.rs` saves and
    loads Wave / CanJoinRaid,
    the entity-NBT tests cover
    the pillager inventory).
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 80 — Bee, verified:**
  - Save/load is Java API-only
    with identical keys; the
    Rust side round-trips them
    all in `mobs/bee.rs`
    (`hive_pos`, `flower_pos`,
    `HasNectar`, `HasStung`,
    `TicksSincePollination`,
    `CannotEnterHiveTicks`,
    `CropsGrownSincePollination`).
    `hasHome` -> `hasRestriction`
    is the item-74 rename.
  - The one real difference is
    the `travellingTicks`
    field initializer:
    1.21.5 seeds it with
    `level().random.nextInt(10)`
    in both travel goals,
    1.21.6 leaves it 0. It
    has no gameplay effect:
    `start()` and `stop()`
    reset it to 0 in both
    versions before any tick,
    and the only other reader
    is the `@VisibleForDebug`
    `getTravellingTicks()`,
    which the Rust port does
    not implement. The two
    level-random draws at
    construction are also not
    ported: the Rust goal
    registration path takes
    only the mob, so the level
    random is not reachable
    there (the port's
    `draw_cooldowns` covers
    the entity-random draws
    only, in vanilla order).
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 81 — Ravager roar to the
  1.21.5 form, ported:**
  - 1.21.6 moved the client's
    half of the roar (40 poof
    particles + the knockback)
    behind entity event 69,
    broadcast by the server;
    1.21.5 runs it directly
    from the client's own tick
    when the roar clock reaches
    10. The port now does the
    1.21.5 form: the server
    roar no longer broadcasts
    69 to LAN guests (its local
    event only drives the
    host's armor-stand push
    under mob_griefing), and
    `tick_remote_entities`
    fires `remote_ravager_roar`
    when a ravager's client
    roar clock reaches 10.
    The event-69 handler stays
    for 1.21.6+ servers only,
    gated on the negotiated
    server version (the pin
    counts as 1.21.5), so a
    newer server's broadcast
    still works and never
    double-fires with the
    client tick.
  - Save/load is Java API-only
    (`AttackTick`, `StunTick`,
    `RoarTick`).
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no ravager unit tests exist;
    the roar path is exercised
    through the client tick).
- **Item 82 — Advancement
  predicates, verified:**
  - `NbtPredicate` is Java
    API-only: 1.21.6 builds the
    comparison tag through
    `ProblemReporter` +
    `TagValueOutput` and names
    the `"SelectedItem"` key
    with a constant, 1.21.5
    through `saveWithoutId` on
    a plain compound and the
    literal key. The output is
    the same (entity NBT, plus
    the selected item for
    players), and the Rust NBT
    predicate matcher already
    tests `SelectedItem` (the
    spyglass test in
    `advancements.rs`).
  - The rest is diagnostics
    and typing with no Rust
    surface: the validation
    path spellings (`"[n]"` /
    `".victims"` / `".entity"`)
    belong to the criterion
    validator the port does
    not implement (item 71),
    and `level()` ->
    `serverLevel()` only
    narrows a type.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 83 — Llama /
  PathfinderMob leash, verified:**
  - The llama's save/load is
    Java API-only (`Variant`,
    `Strength`); the Rust side
    already maps the variant
    component and the Strength
    tag. `supportQuadLeash`
    is 1.21.6-only (the quad
    leash has no Rust surface,
    item 78).
  - The leash hook differs in
    shape but not in result:
    1.21.6's `whenLeashedTo`
    sets home to the holder's
    block with radius
    `(int)leashElasticDistance()
    - 1` = 5, once on attach;
    1.21.5's
    `handleLeashAtDistance`
    restricts to the holder's
    block with radius 5 every
    tick. The Rust leash tick
    already does the 1.21.5
    form (`handle_leash_at_distance`
    sets `ai.home` to the
    holder's block, radius 5,
    each tick, so the home
    follows a moving holder).
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 84 — EditBox to the
  1.21.5 form, other widgets
  verified:**
  - The `TextField` (the
    `EditBox` port) now draws
    the cursor before the
    highlight, as 1.21.5 does
    (1.21.6 reversed them, so
    the selection covers the
    caret), and the 1.21.6-only
    `textShadow` field is gone:
    1.21.5 has no
    `setTextShadow`, its text
    is always shadowed, and
    nothing in the tree ever
    set the field false, so
    the render now passes
    shadow `true` directly.
    Centering never existed
    in the Rust field (1.21.5
    has none either).
  - The text colour stays
    `-2039584`: 1.21.5 spells
    the constant 14737632
    (alpha 0, forced opaque
    by the font), so the
    effective colour is the
    same opaque 0xFFE0E0E0 —
    and the Rust renderer
    drops fully transparent
    text, so the 1.21.5
    spelling would hide the
    value. (Pre-existing gap,
    not a delta: the insert
    cursor is drawn in the
    text colour, where vanilla
    uses the fixed
    `CURSOR_INSERT_COLOR`
    -3092272 in both versions.)
  - The tooltip z parameter
    has no Rust surface (the
    Rust tooltip is deferred
    and drawn last, which is
    what the 1.21.5 z-translate
    achieves); the sprite
    button's alpha argument
    likewise (the Rust sprite
    draw takes no alpha, so
    sprites are opaque as in
    1.21.5). The
    `RenderPipelines` /
    `RenderType` changes are
    Java render-API renames.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no `TextField` unit tests
    exist; the render test
    profile did not finish
    compiling under load).
- **Item 85 — Decorated pot block
  entity, verified:**
  - Pure save/load API changes
    (`ValueInput` / `ValueOutput`
    vs `CompoundTag` +
    `RegistryOps`): the keys
    are identical (`sherds`,
    `item`), with the same
    conditions (sherds only
    when non-empty, the item
    only when no loot table
    applies and it is
    non-empty), and
    `removeComponentsFromTag`
    drops the same two keys
    (`discard` vs `remove`).
    The Rust side already
    canonicalizes the sherds
    and round-trips the item
    compound
    (`block_entity_nbt.rs`,
    with a load/save test).
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 86 — ServerScoreboard,
  verified:**
  - The team-change waypoint
    hooks (`updatePlayerWaypoint`
    / `updateTeamWaypoints`,
    remaking waypoint
    connections on team add /
    remove / change) are
    1.21.6-only: 1.21.5 team
    changes touch no waypoint
    state. The Rust scoreboard
    has no waypoint hooks at
    all — the LAN waypoint
    tracker reads team state
    fresh each tick, so team
    changes propagate without
    them.
  - The `Method` enum
    (`CHANGE`, `REMOVE`) that
    1.21.5 declares in
    `ServerScoreboard` is dead
    code in vanilla itself:
    no usage anywhere in the
    1.21.5 source (1.21.6
    deleted it). No Rust
    surface. The whole
    `net/minecraft/world/waypoints`
    package is 1.21.6-only;
    the locator-bar removal
    is item 108's.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 87 — BakedGlyph, verified:**
  - The glyph geometry is
    identical in both versions
    (italic shear `1 - 0.25 * v`,
    bold thickness 0.1, the
    same UVs) and matches the
    Rust `styled_text` in
    `hud.rs`. The shadow depth
    is already the 1.21.5
    value: the Rust front glyph
    sits at 0.03 in front of
    its shadow (1.21.6 shrank
    it to 0.003).
  - The remaining z deltas are
    unobservable or deliberate:
    the bold copy's +0.001
    (1.21.5 puts it at the same
    z as its glyph) cannot show,
    because the copy and the
    glyph share colour and
    texture, so either depth
    winner draws the same
    pixels; and the
    underline/strikethrough
    lift (+0.01) is the port's
    documented anti-z-fight
    choice, where vanilla
    1.21.5 puts the lines at
    the glyphs' own z. The
    GUI path sets no depth at
    all.
  - The 1.21.6-only additions
    (the bounds methods, the
    `GpuTextureView`, the GUI
    pipeline) have no Rust
    surface, and the effect
    winding change is
    invisible (a solid white
    texel). No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 88 — FallingBlockEntity,
  verified:**
  - The diff is pure Java API
    shape (`ValueOutput` /
    `ValueInput` back to
    `CompoundTag`): the saved
    fields and defaults are
    identical in both versions
    — `BlockState` (default
    sand), `Time` (0),
    `DropItem` (true),
    `HurtEntities` (defaults
    to anvil-ness),
    `FallHurtAmount` (0.0),
    `FallHurtMax` (40),
    `CancelDrop` (false) —
    and the Rust round-trip
    (`entity_nbt.rs`,
    `falling_block.rs`) matches
    every one. The landing
    block-entity transfer is
    the same merge through
    different plumbing.
  - Pre-existing gap, not a
    delta: the port does not
    carry `TileEntityData`
    (`blockData`) on falling
    blocks at all, in either
    version's form.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 89 — FoodData, verified:**
  - Pure Java API shape
    (`ValueInput` / `ValueOutput`
    back to `CompoundTag`) plus
    the `level()` →
    `serverLevel()` rename; the
    tick logic is untouched.
    The saved fields and
    defaults are identical in
    both versions — `foodLevel`
    (20), `foodTickTimer` (0),
    `foodSaturationLevel`
    (5.0), `foodExhaustionLevel`
    (0.0) — and the Rust
    playerdata round-trip in
    `save.rs` plus the
    `FoodData::default` in
    `horizon-world/src/food.rs`
    match every one.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 90 — TamableAnimal, sitting
  parrots stay put on a lead:**
  - The save/load half is pure
    Java API shape (`ValueOutput`
    / `ValueInput` back to
    `CompoundTag`, the null
    Owner check made explicit):
    `Owner` and `Sitting`
    round-trip identically, and
    the Rust `entity_nbt.rs`
    already saves both.
  - The behaviour half is the
    1.21.5-only
    `handleLeashAtDistance`
    override, which 1.21.6
    deleted: a sitting pet is
    not pulled, and its lead
    drops past ten blocks.
    The Rust leash tick already
    had it for wolves and cats
    — but not for parrots,
    which sit when told to
    (`SitWhenOrdered`, the
    same `extra.sitting` flag)
    and are `TamableAnimal`
    too. The sitting check in
    `mobs/leash.rs` now
    covers parrots, with a
    matching test.
  - Drive-by: the diagonal case
    of
    `the_legacy_pull_is_the_signed_square_of_the_unit_direction`
    failed before this item —
    the test expected full f64
    precision for a distance
    that passes through f32
    (vanilla's float). The
    expected unit now comes
    from the truncated value;
    no behaviour change.
  - Verification: `cargo test -p
    horizon-client-lib --lib
    mobs::leash::` 12 passed,
    0 failed (incl. the new
    parrot test); `cargo check
    -p horizon-client-render
    -p horizon-client
    --all-targets` 0 errors /
    0 warnings.
- **Item 91 — PlayerDataStorage,
  verified:**
  - Pure Java API shape
    (`ProblemReporter` /
    `TagValueOutput` /
    `ValueInput` back to a
    plain `CompoundTag`): the
    save/load flow is identical
    in both versions — gzip
    NBT to a temp file, the
    old `.dat` rotated to
    `.dat_old`, atomic
    replace; load tries `.dat`,
    falls back to `.dat_old`,
    then runs the PLAYER
    datafix. The Rust side
    (`save.rs` `write_dat_as`,
    `lan/playerdata.rs`
    `load`) implements exactly
    that flow, including the
    `.dat_old` fallback and
    the safe-replace rotation.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 92 — Level, verified:**
  - `precipitationAt` is a pure
    1.21.6 extraction: the
    logic it holds is
    identical to 1.21.5's
    inlined `isRainingAt`
    (raining, sky visible,
    nothing motion-blocking
    above, biome precipitation
    is rain). The Rust
    `mobs/world.rs` already
    has both forms with the
    exact vanilla logic
    (`precipitation_at` /
    `raining_at`).
  - `onBlockEntityAdded` is a
    dead hook in 1.21.6: an
    empty base method, no
    overrides anywhere,
    called from `LevelChunk`
    but doing nothing. Absent
    in 1.21.5; no behaviour
    either way.
  - `disconnect` lost its
    empty base in 1.21.6 and
    `ClientLevel`'s override
    became `disconnect(Component)`
    — the same disconnect,
    the quit reason now
    passed in instead of
    hardcoded. No Rust
    surface change.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 93 — ResolvableProfile /
  ItemAttributeModifiers,
  verified:**
  - `ResolvableProfile`: the
    1.21.6-only addition is
    `pollResolve` (a
    non-blocking resolve
    check), which has no Rust
    equivalent — the port
    resolves profiles through
    its own cache/async path
    (`native_profiles.rs`).
    The `resolve()` fallback
    (id → profile with name
    or ""; name-only → nil
    UUID) is identical in
    both versions, only
    refactored into a helper
    in 1.21.6.
  - `ItemAttributeModifiers`:
    the per-modifier `Display`
    (default / hidden /
    override) is 1.21.6-only.
    The Rust port reads only
    the 1.21.5 entry shape
    (`type`, `id`, `amount`,
    `operation`, `slot`) and
    the tooltip always
    renders the default form
    — which is exactly
    1.21.5's behaviour, where
    every modifier shows.
    The default-form math
    (base attack damage/speed
    added, x100 for multiply
    ops, x10 for knockback
    resistance, plus/take/
    equals lines) is the
    same logic in both
    versions. No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 94 — BlockItem, verified:**
  - `setBlockEntityData` is pure
    Java API shape
    (`TagValueOutput` back to
    `CompoundTag`): the logic
    is identical — drop
    "id", remove the component
    when the data is empty,
    else tag the entity type
    and set it. The Rust
    `block_entity_item::pick`
    does exactly that
    (`custom_only` strips
    "id" with the other
    metadata keys; the
    component is only set
    when data remains, with
    the type id tagged back
    in). No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 95 — LavaCauldronBlock,
  back to the 1.21.5 content
  test:**
  - 1.21.6 replaced the
    cauldron's burn test with
    the filled-shape test
    (`getEntityInsideCollisionShape`:
    solid shape plus content
    column), which also burns
    anything touching the rim
    or feet. 1.21.5 uses
    `isEntityInsideContent`:
    feet below the lava
    surface (0.9375) and top
    above a quarter block,
    with no horizontal test
    beyond the cell overlap.
    The Rust
    `lava_cauldron_contacts`
    (item entities' burn
    path) had the 1.21.6
    shape; it now does the
    1.21.5 Y-band test, and
    the call-site comments
    drop the 1.21.6
    `LAVA_IGNITE` /
    `StepBasedCollector`
    wording for plain
    `lavaIgnite` then
    `lavaHurt`.
  - The discriminator: an item
    lying flat at the
    cauldron's base (top
    exactly 0.25 up) does not
    burn in 1.21.5, where the
    shape test would burn it
    through a foot. New test
    covers both the inside
    case and the flat case.
  - Verification: `cargo test -p
    horizon-client-lib --lib
    entities::fire_environment_tests`
    2 passed, 0 failed — but
    both early-return here:
    the 1.21.5 tree has no
    `.local` resources on this
    VM, so the assertions run
    in CI, where the resources
    exist. `cargo check -p
    horizon-client-render -p
    horizon-client
    --all-targets` 0 errors /
    0 warnings.
- **Item 96 — ValidationContext /
  ReloadableServerRegistries,
  verified:**
  - Pure Java API shape:
    1.21.6 wrapped the loot
    validation problem
    strings in typed `Problem`
    records (`MissingReference`,
    `ParametersNotProvided`,
    `RecursiveReference`,
    `ReferenceNotAllowed`)
    and `PathElement` params;
    1.21.5 passes plain
    strings. The message
    texts are identical —
    the records' `description()`
    methods return exactly
    the 1.21.5 strings — and
    the validation logic is
    untouched. The Rust port
    has only a partial
    predicate-graph check
    (`predicate_graph.rs`,
    explicitly not a port of
    `ValidationContext`),
    whose diagnostic texts
    match both versions.
  - `ReloadableServerRegistries`:
    the log iteration differs
    in shape only, same
    output; 1.21.5's extra
    `getKeys` helper feeds
    only `/loot` and `/item`
    command suggestions,
    whose content is identical
    in both versions.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 1 warning
    (an unused `VoxelShape` import
    left by item 95's shape removal;
    fixed in the follow-up commit
    below).
- **Item 95 follow-up — unused import:**
  item 95 removed the only use of
  `VoxelShape` in `entities.rs`;
  the import is dropped, so the
  check is warning-free again.
- **Item 97 — EntityPredicate /
  SimpleCriterionTrigger,
  verified:**
  - `EntityPredicate`: two
    `level()` → `serverLevel()`
    renames, same object both
    ways. The Rust matcher
    (`advancements.rs`
    `entity()`) just receives
    the world; the rename is
    invisible to it.
  - `SimpleInstance.validate`:
    the validation path
    spelling changes from
    `"player"` (1.21.6) to
    `".player"` (1.21.5).
    No criterion validator
    exists in the Rust tree
    (as items 71 and 82
    noted), so validation-path
    spellings have no surface.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 98 — RecipeBookSettings,
  verified:**
  - A Java refactor with
    identical formats: the
    wire codec is 8 booleans
    in type order in both
    versions (the Rust packet
    codec matches), and the
    NBT keys are the same.
    The one real difference
    is the NBT write:
    1.21.5 writes all eight
    flags always, while
    1.21.6's codec omits the
    false ones — and the Rust
    `Settings::write` already
    does the 1.21.5 form
    (its doc comment calls
    out the 1.21.6 difference
    explicitly). The EnumMap
    vs four fields, and the
    `equals`/`hashCode`
    override, are internal
    Java details.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 99 — ARGB, verified:**
  - 1.21.6 added two methods
    1.21.5 lacks. `color(float,
    int)` (float-alpha
    variant) is a Java
    convenience with no Rust
    equivalent and no
    behaviour. `setBrightness`
    (HSV brightness) is dead
    code in vanilla 1.21.6
    itself — no callers
    anywhere in the 1.21.6
    source.
  - The Rust port does have a
    `set_brightness`
    (`locator_bar.rs`), but
    its only caller is the
    locator bar's waypoint
    dimming — a 1.21.6
    feature whose removal
    belongs to item 108, so
    it stays here. Everything
    else in ARGB is identical
    in both versions.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 100 — SpriteSourceList /
  DownloadedPackSource,
  verified:**
  - `SpriteSourceList`: 1.21.6
    parses atlas definitions
    with `StrictJsonParser`,
    1.21.5 with Gson's plain
    `JsonParser`. Only
    malformed pack JSON could
    tell them apart; valid
    definitions parse
    identically, and the Rust
    decoder (`serde_json`,
    whole file fails on a
    bad one) matches vanilla's
    fail-the-file shape.
  - `DownloadedPackSource`:
    pure getter renames
    (`name()` → `getName()` and
    friends). The Rust
    `server_packs/download.rs`
    sends the same header
    values (username, UUID,
    version, version id,
    pack format, User-Agent).
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 101 — ItemStackRenderState,
  verified:**
  - 1.21.6 added `animated`,
    `oversizedInGui`, the
    model-identity list and
    the cached model bounding
    box. The GUI-side uses
    (atlas-position caching,
    animated tracking, the
    oversized-item renderer)
    belong to 1.21.6's new
    `GuiRenderer`, which has
    no Rust equivalent — the
    Rust GUI renderer has no
    oversized-item or
    model-identity concepts.
  - The bounding box itself
    is not a delta: 1.21.5's
    `ItemEntityRenderer`
    computes the same box
    with its own private
    `calculateModelBoundingBox`
    (same `visitExtents`
    computation); 1.21.6 only
    moved it onto the render
    state with caching. The
    Rust dropped-item
    renderer already uses
    the model's extents
    (`state.bounding_box()`)
    for the same purpose.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 102 — AABB, verified:**
  - 1.21.6 added two
    convenience methods.
    `intersects(BlockPos)`
    delegates to the existing
    6-double intersects; no
    Rust equivalent is
    needed. `distanceToSqr(AABB)`
    has exactly one vanilla
    caller, the splash
    potion — and item 22
    already ported that to
    the 1.21.5 form
    (feet-to-feet distance,
    direct hit at full
    strength). The Rust
    `Aabb` carries the same
    formula as
    `distance_squared_to_box`,
    used only by the geometry
    parity probe, not by
    gameplay. No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 103 — ServerStatsCounter,
  verified:**
  - The parse difference is
    plumbing only: 1.21.6 uses
    `StrictJsonParser`, 1.21.5
    a non-lenient `JsonReader`
    plus `Streams.parse` —
    both strict JSON parsing
    with the same failure
    outcome (log and keep
    empty stats), the same
    STATS datafix (default
    version 1343) and the
    same codec over the
    `"stats"` object. The
    `toJson` change is a pure
    getter rename; the same
    DataVersion is written.
    The Rust `game/stats.rs`
    (`serde_json` strict
    parse, `"stats"` object,
    `DataVersion` from the
    version constant) matches
    the shared behaviour.
    Pre-existing gap, not a
    delta: the Rust read does
    not run the STATS
    datafix. No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 104 — ServerEntity,
  verified:**
  - The teleport condition:
    1.21.6 always
    teleport-syncs entities
    with
    `getRequiresPrecisePosition`,
    which only the happy
    ghast sets (a 1.21.6-only
    mob). The Rust tracker
    has a `precise` flag, but
    nothing in production
    ever sets it true (only
    its test does), so the
    behaviour is already the
    1.21.5 form.
  - Pairing motion: 1.21.5's
    `sendPairingData` also
    sends `SetEntityMotion`
    (the last broadcast
    velocity) for non-living
    `trackDelta` entities;
    1.21.6 and the Rust
    `pairing` do not. It is
    unobservable: the spawn
    packet already carries
    the current velocity, and
    the last broadcast
    velocity is kept within
    the 1e-7 broadcast
    threshold of it by the
    regular tracking tick —
    which is also why the
    pairing motion only
    applies to `trackDelta`
    types. (The fall-flying
    line in 1.21.5's pairing
    is dead: it sets a flag
    consumed only by a check
    that excludes living
    entities.) The main-branch
    motion block is identical
    in both versions and the
    Rust tracker matches it.
    No code change.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (no-op re-check).
- **Item 105 — CloudRenderer /
  RenderPipelines /
  SectionOcclusionGraph, clouds
  to the 1.21.5 form:**
  - The 1.21.6 cloud renderer
    is a GPU rewrite (UTB face
    buffer, CloudInfo UBO,
    Cloud Distance option,
    default 128 chunks); the
    visual algorithm is the
    same in both versions
    (12-block cells, 4-block
    height, 0.03 drift, the
    same face colours and
    camera-relative culling)
    except the mesh extent
    and the fog. 1.21.5 builds
    a fixed 65x65-cell square
    (-32..=32); the Rust mesh
    iterated 1.21.6's circle
    instead. `CellIter` now
    walks the full square,
    still nearest-first so the
    single depth-writing pass
    occludes far faces behind
    near ones (the job
    vanilla's 1.21.5-only
    `CLOUDS_DEPTH_ONLY`
    pre-pass does).
  - The fog: 1.21.5's
    `rendertype_clouds.fsh`
    (read from the 1.21.5
    client jar, sha1-verified)
    is `linear_fog` over the
    render-distance pair the
    clouds pass inherits from
    the particles pass — the
    colour blends towards the
    fog colour between
    rd-clamp(rd/10, 4, 64)
    and rd, cylindrical
    distance, alpha untouched.
    1.21.6 fades the alpha
    linearly to its cloud fog
    end (2048 by default).
    The Rust shaders did the
    1.21.6 form, so clouds
    showed at ~80% alpha past
    the render distance where
    vanilla 1.21.5 has them
    fully blended into the
    sky. The cloud branch
    (`entity.wgsl`, overlay
    4), the OIT cloud colour
    and the HDR cloud mark
    now use the 1.21.5 form;
    the vertex shader gives
    clouds both fog distances
    (they are no longer
    exempt from the
    render-distance fog).
  - `FogData.cloud_end` is
    kept, but only as the
    pack-shader fog block's
    value: 1.21.5 has no
    `visual/cloud_fog_end_distance`
    dimension field at all
    (verified against the
    1.21.5 server jar — no
    `dimension` registry,
    only `dimension_type`),
    and nothing in the vanilla
    cloud path reads the slot
    any more. The fog test is
    renamed to say so.
  - RenderPipelines is the
    1.21.6 uniform-buffer
    refactor (DynamicTransforms
    / Projection / Fog /
    Globals UBOs vs 1.21.5's
    individual uniforms) plus
    1.21.5-only pipelines
    (`GUI_OVERLAY`,
    `GUI_GHOST_RECIPE_OVERLAY`,
    `GUI_TEXTURED_OVERLAY`,
    `CLOUDS_DEPTH_ONLY`) —
    GPU plumbing with no Rust
    surface; the Rust renderer
    is its own wgpu
    implementation and layers
    the GUI its own way.
    SectionOcclusionGraph is a
    rename (`CompiledSection`
    to `SectionMesh`):
    identical traversal.
    Pre-existing gaps, not
    deltas: the snap margin
    shows a ring of clouds
    past vanilla's 32-cell
    window at render distance
    25-32 chunks (noted in
    `clouds.rs`), and the
    cloud parity oracle still
    calls 1.21.6's
    `buildMesh`/`buildExtrudedCell`
    signatures, so it cannot
    run against the 1.21.5 jar
    (the harness is not in the
    1.21.5 subset).
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings;
    WGSL validated with naga 30.0.1
    (entity + oit_clouds parse and
    validate); the square iterator
    checked standalone (radii 0-36:
    full square, nearest-first,
    unique, corners included).
- **Item 106 — SkullBlockRenderer /
  ShulkerBoxRenderer / Font /
  LayerDefinitions /
  HangingSignRenderer, Font's
  adjustColor:**
  - Font: 1.21.6 rewrote text
    preparation (`PreparedText` /
    `GlyphVisitor`, bounds for the
    new GuiRenderer); 1.21.5 draws
    through `StringRenderOutput`.
    The one behavioural rule the
    rewrite dropped is
    `adjustColor`: 1.21.5 forces a
    default colour with alpha
    under 4 to opaque (callers
    pass plain RGB colours), in
    `drawInBatch` and in
    `drawInBatch8xOutline` for
    both colours. The Rust
    `styled_text` used the alpha
    as-is, and `screen_draw::text`
    dropped alpha-0 draws
    outright — the opposite of
    vanilla. `styled_text` now
    adjusts the default and the
    outline colour at the top
    (the per-style colours take
    the default's alpha, as
    `getTextColor` does), and the
    drops are gone. The chat's
    own alpha-3 cut (lines at 3
    or below draw nothing) is
    vanilla's guard against the
    rule and is untouched; the
    toast fade never drops below
    alpha 64.
  - The depths already match
    1.21.5 (item 87): shadow
    0.03 (1.21.6: 0.003), effect
    lines +0.01 (1.21.6: 0.001),
    bold +0.001; the line rects
    (strike 3.5..4.5, underline
    8..9) match, the top/bottom
    swap in the diff being a
    winding-only difference.
    `ALPHA_CUTOFF` (8) is dead
    in 1.21.5 itself (no users).
    The `inverseDepth=false`
    path has one caller
    (MapRenderer) and no Rust
    surface — the port draws no
    map decoration text.
  - Skull: the 1.21.6 split into
    `getPlayerSkinRenderType` /
    `getSkullRenderType` is a
    pure refactor; the Rust
    renderer already takes the
    translucent path for a head
    with a resolved profile and
    cutout otherwise. Shulker:
    the `prepareModel` extraction
    is a pure refactor — the Rust
    draw has the same 0.9995
    scale, facing rotation and
    lid animation;
    `getExtents` is 1.21.6-only
    GUI bounding-box plumbing.
    LayerDefinitions only drops
    the happy ghast models (a
    1.21.6-only mob; no trace of
    it in the Rust tree).
    HangingSignRenderer only
    narrows two members to
    private. No code change for
    any of those.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings;
    `cargo test -p horizon-client-render
    --lib screen_draw::scissor_tests::
    text_with_alpha_under_four_draws_opaque`
    1 passed.
- **Item 107 — FontTexture / glyph
  providers / Font, unihex
  `size_overrides` required:**
  - The Font.java section is the
    same diff as item 106 (the
    splitter put the file in both
    items) — covered there,
    including adjustColor.
  - The one new delta is
    `UnihexProvider$Definition`:
    1.21.6 made `size_overrides`
    optional with an empty
    default; 1.21.5's codec
    requires the field, so a
    definition without it fails
    to load and the provider is
    skipped. The Rust loader
    treated a missing field as
    no overrides (the 1.21.6
    form). `load_set` now skips
    a unihex provider without
    the field, like the other
    providers' load failures.
  - The rest is GPU plumbing:
    FontTexture's texture view
    and `createTexture` arity,
    and the glyph uploads'
    `writeToTexture` mipmap
    argument — same pixels, same
    0.01 UV inset. No code
    change for those.
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings;
    `cargo test -p horizon-client-render
    --lib font::tests::
    a_unihex_provider_without_size_overrides_is_skipped`
    1 passed.
- **Item 108 — WaypointStyle /
  `world/waypoints` /
  `Vec3.rotateClockwise90`,
  the locator bar removed:**
  - 1.21.5 has no locator
    bar: `WaypointStyle.java`
    and the `world/waypoints`
    package are 1.21.6-only,
    and the publish had
    already unwired the
    `ClientboundTrackedWaypoint`
    packet from the
    clientbound enum, so the
    bar never drew in
    1.21.5. The client bar
    (`locator_bar.rs`: the
    styles, the waypoint
    view, `set_brightness`)
    and the LAN tracker
    (`lan/waypoints.rs`,
    `game/lan_waypoints.rs`,
    `game/locator_bar.rs`)
    are deleted, with the
    `Game` fields, the
    `LanHost` fields, the
    `WaypointStyles` load
    and reload, the five
    `clear_waypoints`
    calls, and the
    `TrackedWaypoint` codec
    and its types in
    `horizon-protocol`.
  - `Vec3.rotateClockwise90`
    is 1.21.6-only too: the
    Rust method, the
    geometry probe's
    `"clockwise"` line and
    the oracle's call are
    gone (the oracle would
    not compile against the
    1.21.5 jar otherwise).
  - The shared test client
    `LocatorBarClient` kept
    a waypoint branch that
    would not compile
    against the 1.21.5 jar
    either: the import, the
    `instanceof` branch and
    the `waypoint` reader
    are removed, so the
    dedicated-server
    harnesses that use it
    still build. The
    locator-bar harness
    itself
    (`tools/test_locator_bar_parity.py`)
    stays in the tree but
    cannot run on 1.21.5:
    vanilla sends no
    waypoint packets.
  - Deliberately kept for
    their own items: the
    `locator_bar` game rule
    (item 131), the
    `/waypoint` command
    (items 36/52/145) and
    the
    `waypoint_transmit_range`/
    `waypoint_receive_range`
    attribute hooks (items
    13/145/225/260/264).
  - Docs updated:
    architecture (HUD row),
    agent-control (frozen
    HUD list), native-mods
    (`contextual_info_bar`
    is the jump bar or the
    experience bar),
    resource-packs (the
    `waypoint_style/` row).
  - Verification: `cargo check -p
    horizon-client-render -p horizon-client
    --all-targets` 0 errors / 0 warnings
    (run at `-j 1`: the default parallel
    check OOM-killed both horizon-client
    rustc processes under 3-stream load).
- **Item 109 — SimpleContainer /
  Villager, the `ValueInput` /
  `ValueOutput` save API back
  to `CompoundTag`:**
  - A pure API refactor, no
    behavioural delta: 1.21.6
    routes container and
    entity saves through
    `ValueInput` /
    `ValueOutput`
    (`fromItemList`,
    `storeAsItemList`,
    `addAdditionalSaveData
    (ValueOutput)`); 1.21.5
    uses `CompoundTag`
    directly (`fromTag`,
    `createTag`,
    `addAdditionalSaveData
    (CompoundTag)`). The
    serialized form and the
    field lists are identical
    in both versions.
  - The Rust port already
    matches the 1.21.5 form:
    `load_container`
    (`mobs/villager.rs`)
    parses each compound and
    drops failures silently
    (1.21.5's
    `ItemStack.parse`
    flatMap), `addItem`s
    each stack at the type's
    limit, and drops what
    does not fit; the
    inventory vec never
    holds empty stacks, so
    the write side matches
    `createTag`'s non-empty
    rule. The villager
    save/load covers every
    1.21.5 field:
    VillagerData, FoodLevel,
    Gossips, Xp,
    LastRestock,
    LastGossipDecay,
    RestocksToday and the
    inventory
    (`entity_nbt.rs`,
    `mobs/villager.rs`
    `read_nbt` /
    `write_nbt`).
  - Only the stale 1.21.6
    method names in comments
    changed (`fromItemList`
    to `fromTag`, in
    `villager.rs`,
    `entity_nbt.rs` and
    `persist.rs`).
  - Pre-existing gaps, the
    same in 1.21.6 (not this
    item's delta): the port
    does not carry
    `AssignProfessionWhenSpawned`,
    and does not refresh the
    brain on load. Two
    oracles still use the
    1.21.6 save API and
    would not compile
    against the 1.21.5 jar
    (`VanillaArrowOracle`'s
    `ValueInput` /
    `ValueOutput` overrides,
    `VanillaChestLoadOracle`'s
    `TagValueInput`) —
    publish debt, like the
    cloud oracle; neither
    harness is in the 1.21.5
    subset.
  - Verification: `cargo check -j 1
    -p horizon-client --lib` 0 errors
    / 0 warnings.
- **Item 110 — op / deop / pardon
  / pardon-ip / place / save-all
  / summon / teleport /
  whitelist, `hasPermission`
  inlined; `/version` gone:**
  - A pure refactor, no
    behavioural delta: 1.21.6
    spells the requirement
    `Commands.hasPermission(N)`,
    1.21.5 the lambda
    `$$0x -> $$0x.hasPermission(N)`;
    the predicate is the
    same. The summon
    suggestion change is the
    same kind of thing:
    1.21.6 wraps the provider
    in a generics cast,
    1.21.5 passes it
    directly.
  - The Rust tree's levels
    match 1.21.5 exactly:
    op, deop, pardon,
    pardon-ip and whitelist
    at 3 (ADMINS),
    save-all at 4 (OWNERS),
    place, summon and
    teleport (and the `tp`
    alias) at 2
    (GAMEMASTERS)
    (`horizon-command`
    `vanilla.rs`,
    `tree_extra.rs`,
    `tree_world.rs`); summon
    suggests
    `Suggests::SummonableEntities`,
    the same provider.
  - The one real delta is
    `VersionCommand.java`,
    removed in 1.21.5: the
    publish already took
    `/version` out of the
    tree (the marker comment
    in `game/commands/
    extra.rs`); no `/version`
    remains anywhere in the
    Rust tree. No code
    change.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client --lib`
    passed at item 109.
- **Item 111 — Display /
  BlockDisplay / ItemDisplay
  / TextDisplay, save/load
  back to `CompoundTag`:**
  - The same
    `ValueInput` /
    `ValueOutput` refactor
    as item 109: 1.21.5
    reads and writes the
    display fields from a
    `CompoundTag`, building
    the `RegistryOps`
    context explicitly for
    the block-state, item
    and text codecs. The
    serialized form and the
    field lists are
    identical in both
    versions.
  - The Rust port already
    implements 1.21.5's
    `readAdditionalSaveData`
    exactly, in
    `text_display.rs`
    (`display_data` /
    `text_display_data`) and
    `game/lan_entities.rs`:
    start delay, durations,
    teleport clamped 0..59,
    the transformation
    (four parts or a
    MATRIX4F, else
    identity), billboard,
    brightness packed
    block<<4 | sky<<20,
    view range, shadow,
    width, height, glow
    colour override; text,
    line width 200,
    background
    0x40000000, opacity -1,
    the shadow / see-through
    / default-background
    flags and the alignment
    bits; block state
    defaulting to air, item
    plus the display
    context (0..9, the enum
    unchanged between the
    versions).
  - The one nuance is in
    TextDisplay's text
    read: 1.21.5 sets the
    text to empty when the
    codec parses the tag to
    null, 1.21.6 leaves it
    unset. Unobservable: a
    freshly loaded display
    defaults to empty
    either way, and the
    read runs only at load.
  - The write side
    (`addAdditionalSaveData`)
    has no Rust surface:
    displays are kept
    unsimulated and their
    compounds pass through
    unchanged. No code
    change.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client --lib`
    passed at item 109.
- **Item 112 — Chicken / Pig /
  AbstractFish / Salmon,
  save/load plus variant
  selection:**
  - The save/load half is the
    same `ValueInput` /
    `ValueOutput` refactor
    as items 109/111: the
    fields are identical in
    both versions
    (IsChickenJockey,
    EggLayTime, the variant
    name, FromBucket, the
    salmon `type`), and the
    Rust port already reads
    and writes them all
    (`entity_nbt.rs`,
    `mobs/riders.rs`): the
    variant as the
    `minecraft:<name>`
    string under `variant`,
    EggLayTime only when
    present, the salmon
    `type` defaulting to
    medium (vanilla's
    DEFAULT).
  - The selection half is
    the generic extraction:
    1.21.6 calls
    `VariantUtils.
    selectVariantToSpawn`,
    1.21.5 the per-type
    `ChickenVariants` /
    `PigVariants.
    selectVariantToSpawn`.
    The Rust port already
    implements the 1.21.5
    per-type form:
    `VariantRegistry::select`
    (`mobs/finalize.rs`)
    takes the entries of the
    highest priority whose
    condition holds and
    picks one at random,
    and `finalize` calls it
    for pig, cow and chicken
    before the sound variant
    and the ageable step.
  - Pre-existing gap, the
    same in 1.21.6: the port
    writes `IsChickenJockey`
    but never reads it back,
    so a loaded chicken
    jockey resumes laying
    eggs (vanilla jockey
    chickens never lay).
    Noted, not this item's
    delta. No code change.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client --lib`
    passed at item 109.
- **Item 113 — SectionCompiler,
  chunk layers keyed by
  `RenderType`; off-screen
  block entities stay in the
  section list:**
  - The layer half is a pure
    refactor: 1.21.6 keys
    the section's meshes by
    the `ChunkSectionLayer`
    enum, 1.21.5 by
    `RenderType` directly
    (and `RenderSectionRegion`
    is still
    `RenderChunkRegion`).
    Same layers, same
    translucent sort. The
    Rust mesher has its own
    layer buckets
    (`mesher::OPAQUE_LAYER`
    and friends) driven by
    the 1.21.5 block tables;
    no `ChunkSectionLayer`
    equivalent exists to
    remove. One comment in
    `gfx/pack_atlas.rs`
    named the 1.21.6 enum;
    it now names
    `RenderType.solid()`.
  - The block-entity half
    is real but already
    matches 1.21.5: 1.21.6
    drops off-screen block
    entities from the
    section's list, 1.21.5
    keeps them in it (and
    also in the global
    list). The Rust mesher
    pushes every
    block-entity block into
    the section's list with
    no off-screen filter,
    and the game never
    culls the off-screen
    set — beacon, end
    gateway, structure
    block, test instance
    block (`game.rs`
    `off_screen`, matching
    vanilla 1.21.5's
    `shouldRenderOffScreen`
    overrides) — with the
    end gateway kept at long
    range too. The per-entity
    argument 1.21.5's
    `shouldRenderOffScreen`
    takes is unused by every
    override (all constant
    true), so the 1.21.6
    no-arg form changes
    nothing observable.
  - Verification: `cargo check -j 1
    -p horizon-client-render --lib`
    0 errors / 0 warnings.
- **Item 114 — CreativeModeInventoryScreen:**
  - GPU plumbing only for the
    pipeline swap
    (`RenderPipelines` to
    `RenderType`): the Rust
    creative screen draws
    with its own renderer.
    No surface.
  - The colour spellings
    are the alpha-0 forms
    1.21.5 writes (search
    text 16777215, tab title
    4210752) where 1.21.6
    writes the opaque ones
    (-1, -12566464). Both
    render identically:
    1.21.5's `adjustColor`
    forces alpha under 4 to
    opaque (item 106). The
    Rust screen uses the
    opaque spellings
    (`u32::MAX` for the
    search box, rgba(64, 64,
    64, 255) for the title),
    which the Rust renderer
    needs — it drops fully
    transparent text (item
    84).
  - The draw order is
    already the 1.21.5
    form: effects after the
    panel and before the
    tooltips
    (`draw_inventory_effects`
    at the end of
    `draw_creative`, then
    the tab and trash
    tooltips), where 1.21.6
    draws the effect icons
    before the screen. The
    tab icon position
    (x+5, top +9 / bottom
    +7) matches 1.21.5
    exactly, and Rust's
    `draw_item` draws the
    decorations 1.21.5's
    `renderItemDecorations`
    call adds (1.21.6 drops
    them) — invisible for
    vanilla count-1
    undamaged icons either
    way. The hotbar save
    calls `say_now`, the
    1.21.5 narrator name
    (1.21.6 renamed it
    `saySystemNow`).
  - No code change.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client-render --lib`
    passed at item 113.
- **Item 115 — FishingHook:
  no interpolation, gravity
  out of water even on the
  ground:**
  - 1.21.6 gave the hook a
    `LinearInterpolationHandler`
    (three steps towards a
    server update) and calls
    `interpolate()` in
    `tick`; 1.21.5 has
    neither. The Rust port
    had the 1.21.6 form
    (`HookLerp`, three steps,
    carried along by the
    predicted movement). It
    is removed: `Entity.
    getInterpolation` is null
    for a 1.21.5 hook, so
    `moveOrInterpolateTo`
    snaps it, and both
    `move_to`s now set the
    position at once. The
    client prediction
    (`client_hook_step`
    between tracker updates)
    stays — vanilla 1.21.5's
    client simulates the hook
    locally too. Test:
    `a_server_update_snaps_a_remote_hook`.
  - Gravity: 1.21.6 applies
    it only out of water AND
    off the ground AND
    unhooked; 1.21.5
    whenever out of water.
    The hooked condition is
    dead in both versions
    (the state machine
    returns before the line
    whenever a hook is set),
    and a flying hook's
    motion is zeroed after
    the move either way, so
    the only observable
    case is a bobbing hook
    resting on dry ground,
    which now keeps a
    sinking velocity as in
    1.21.5. Both copies of
    the tick
    (`tick_fishing`,
    `client_hook_step`)
    take the 1.21.5 form.
  - Save/load is empty in
    both versions; the API
    change has no surface.
  - Verification: `cargo check -j 1
    -p horizon-client --lib --tests`
    0 errors / 0 warnings;
    `cargo test -j 1 -p horizon-client
    --lib game::fishing::tests::
    a_server_update_snaps_a_remote_hook`
    1 passed. (The first test run
    was SIGKILLed mid-codegen by
    a runtime restart; the retry
    passed.)
- **Item 116 — PlayerTabOverlay /
  EntitySelectorOptions (nbt):**
  - Tab list: the narrator
    call is `sayNow` in
    1.21.5 (`saySystemNow`
    is the 1.21.6 rename) —
    the Rust call in
    `game/scoreboard.rs`
    already says `say_now`.
    The ping icon's z+100
    lift in 1.21.5 (drawn
    above the row) matches
    the Rust draw list,
    which pushes the ping
    sprite after the row's
    name and score. The
    score colour 16777215
    and the hearts number's
    red-green gradient are
    alpha-0 spellings that
    1.21.5's `adjustColor`
    forces opaque (item
    106); the Rust port
    writes the opaque forms
    (`0xFFFF_FFFF`, the
    gradient OR'd with
    `0xFF00_0000`). The
    `RenderPipelines` to
    `RenderType` swap is GPU
    plumbing. No code
    change.
  - Selector `nbt`: the
    same `ValueInput` /
    `ValueOutput` refactor
    as items 109/111 — the
    1.21.5 form builds the
    entity's compound with
    `saveWithoutId` and
    adds `SelectedItem` for
    a player, exactly as
    `NbtPredicate.
    getEntityTagToCompare`
    does in 1.21.5 (the
    helper exists in both
    versions; 1.21.6's
    selector just routes
    through `TagValueOutput`
    instead). The Rust
    `player_nbt`
    (`game/commands/
    data.rs`) implements
    it: the saved tag plus
    the held stack with
    its `Slot` key removed.
    No code change.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client --lib --tests`
    passed at item 115.
- **Item 117 — Allay /
  FlyingPathNavigation:**
  - `Allay.travel` is the
    extraction in reverse:
    1.21.6 calls
    `travelFlying`, 1.21.5
    inlines it (0.02 in
    water and lava, the
    move-control speed in
    the air; drag 0.8 /
    0.5 / 0.91). The Rust
    port already implements
    the 1.21.5 inline form
    (`mobs/allay.rs`
    `travel` via
    `travel_flying_speeds`,
    whose comments cite
    the 1.21.6 extraction).
    The save/load half is
    the usual
    `ValueInput` /
    `ValueOutput` refactor
    with identical fields:
    Inventory, the
    vibration `listener`
    and `DuplicationCooldown`
    are all read and
    written (`entity_nbt.rs`).
  - Flying navigation: the
    1.21.6
    `canNavigateGround`
    override (false) only
    restates what 1.21.5's
    `GoalUtils.
    hasGroundPathNavigation`
    already says — the Rust
    `can_navigate_ground`
    (`mobs/doors.rs`)
    returns false for
    `NavKind::Flying` in
    both readings.
    `setCanOpenDoors` in
    1.21.5 is a plain
    delegate to the node
    evaluator; the Rust
    `Nav.can_open_doors`
    is a field no flying
    mob sets, and no
    vanilla flying mob
    opens doors.
    `isStableDestination`
    is identical in both
    versions (the Rust
    flying case checks
    `STURDY_TOP`).
  - No code change.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client --lib --tests`
    passed at item 115.
- **Item 118 — ZombieVillager:
  the random profession moves
  to the constructor:**
  - 1.21.6 draws the random
    profession in
    `finalizeSpawn` (after
    the biome type, before
    the zombie's own
    finalize); 1.21.5 draws
    it in the constructor,
    and `finalizeSpawn`
    only sets the type by
    biome. The Rust port
    already models the
    1.21.5 form:
    `constructor_profession`
    (`mobs/zombie_villager.rs`)
    draws from a fresh
    random derived from the
    mob's state, leaving
    the finalize sequence
    untouched, and
    `finalize` calls it
    after `villager_type`
    (which draws nothing)
    and before the zombie
    step. A load without
    saved VillagerData gets
    the constructor draw
    too (`read_nbt`).
  - The save/load half is
    the usual `ValueInput` /
    `ValueOutput` refactor
    with identical fields:
    VillagerData, Offers,
    Gossips, ConversionTime
    and ConversionPlayer
    are all read and
    written, and a
    ConversionTime other
    than -1 resumes the
    cure. No code change.
  - Pre-existing nit: the
    zombie-villager write
    omits an empty Gossips
    list where vanilla
    writes it (the villager
    write always writes
    it); the loaded state
    is identical either
    way.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client --lib --tests`
    passed at item 115.
- **Item 119 — TicketStorage:
  `purgeStaleTickets` counts
  every ticket down:**
  - 1.21.6's purge takes the
    `ChunkMap` and holds a
    simulation ticket while
    its chunk's holder is
    not ready for saving
    (`canTicketExpire`);
    1.21.5's takes nothing
    and decrements every
    ticket. The Rust port
    had the 1.21.6 gate
    (`tick_portal`'s
    `can_expire` callback,
    `portal_ticket_can_expire`
    in `game.rs`). Both are
    gone: `tick_portal`
    counts every `PORTAL`
    ticket down each tick.
    Test:
    `portal_tickets_count_down_unconditionally`.
  - The parity harness
    tested the 1.21.6 gate
    too, and its oracle
    called the 1.21.6
    `purgeStaleTickets(map)`
    signature, which does
    not exist on the 1.21.5
    jar. All three are
    ported: the oracle now
    calls `purgeStaleTickets()`
    with no chunk-map or
    holder allocation, the
    fixture's `tick` op
    drops its `loading`
    set, and the harness
    drops the `loading`
    cases (the 301-tick
    hold case is now a
    plain timeout case).
    `removeTicketIf`'s
    arity change has no
    Rust surface, and
    `deactivateTicketsOnClosing`
    behaves the same in
    both versions.
  - Verification: `cargo check -j 1
    -p horizon-client --lib --tests`
    and `-p horizon-client-lib
    --example client_probe` 0 errors
    / 0 warnings; `cargo test -j 1
    -p horizon-client-lib --lib
    ticket_levels::tests::
    portal_tickets_count_down_unconditionally`
    1 passed.
- **Item 120 — Horse, skeleton
  horse, chested horse, trader
  llama save/load verified:**
  - The save/load delta is the
    usual ValueInput/ValueOutput
    refactor with identical
    fields: Horse `Variant`
    (packed colour | marking
    << 8), SkeletonHorse
    `SkeletonTrap` /
    `SkeletonTrapTime`,
    chested horse
    `ChestedHorse` + slotted
    `Items`, trader llama
    `DespawnDelay` (default
    47999). The Rust port
    reads and writes all of
    them (`entity_nbt.rs`,
    `mobs/riders.rs`,
    `mobs/llama.rs`), and
    `decode_items` matches
    the 1.21.5 read exactly
    (missing `Slot` defaults
    to 0, out-of-range slots
    dropped, unparseable
    stacks empty).
  - `AbstractChestedHorse`'s
    `getQuadLeashOffsets` is
    1.21.6-only; the port's
    leash system is already
    the 1.21.5 single-leash
    form, with no quad-leash
    code anywhere to remove.
  - Pre-existing nit, same in
    both versions: a chested
    horse with an empty chest
    writes no `Items` key
    where vanilla writes an
    empty list; the read
    treats both the same.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client --lib --tests`
    passed at item 119.
- **Item 121 — Item model
  resolver: oversized-in-GUI
  removed, skull fixup and
  dialog registries verified:**
  - 1.21.6's `appendItemLayers`
    sets `oversizedInGui` from
    the item properties; the
    concept does not exist in
    1.21.5 at all (verified
    against the 1.21.5 jar:
    `ClientItem.Properties`
    has only
    `handAnimationOnSwap`,
    `ItemStackRenderState`
    has no oversized field).
    The Rust port carried
    the whole concept
    (`ClientItem` /
    `ResolvedItem` /
    `ItemRenderState`
    fields, the JSON decode,
    the GUI slot clip) — item
    101's entry claimed it
    had none, which was
    wrong. All of it is
    gone: the GUI item draw
    now clips only by the
    caller's scissor, as
    1.21.5's `GuiGraphics.renderItem`
    does (1.21.6's
    `GuiRenderer` clips each
    item to its 16x16 slot).
    Item 138 owns the
    `ClientItem.Properties`
    record itself; the
    property half is
    removed here because
    the call site cannot
    go without it.
  - The item-model oracle
    and fixture drop the
    `oversized` output
    together, and the
    harness's bool-coercion
    cases move to
    `hand_animation_on_swap`.
  - `fixupSkullProfile` is
    1.21.5-only (an
    unresolved skull
    profile is dropped and
    re-set once resolved).
    The Rust skull-item path
    already has its
    observable behaviour:
    `head_skin` resolves
    the profile on every
    draw and falls back to
    the default skin while
    it is pending, so no
    fixup hook is needed.
  - `appendModelIdentityElement`
    (1.21.6) has no Rust
    surface: the port has
    no model-identity
    concept. The dialog
    registries and packets
    are already gone (the
    mechanical publish);
    the game-side dialog
    types stay, local-only,
    per that publish's
    decision.
  - Pre-existing harness
    debt, same as 1.21.6
    (whose published
    profile has item_model
    failing): the oracle
    still uses the
    later-version
    `ItemOwner` type and
    `swapAnimationScale`,
    neither of which exists
    on the 1.21.5 jar, so
    it still does not
    compile there. The
    Rust `swap_animation_scale`
    stays too: the game
    uses it for the hand
    swap, defaulting to
    1.0, which is 1.21.5's
    behaviour for all
    vanilla content.
  - Verification: `cargo check -j 1
    -p horizon-assets --all-targets
    -p horizon-client --lib --tests`
    0 errors / 0 warnings.
- **Item 122 — Sign edit screen:
  wall signs sit 35 lower;
  `getDarkColor` verified:**
  - 1.21.5's `SignEditScreen.offsetSign`
    adds 35 for a non-standing
    sign, so a wall sign's
    text and model centre on
    y=125, not y=90;
    1.21.6's `getSignYOffset`
    returns 90 for every
    sign. The Rust editor
    had the 1.21.6 form
    (90 for any non-hanging
    sign). It now centres
    wall signs at 125, in
    both the draw
    (`draw_sign_editor`,
    text and sprite) and
    the click hit-test
    (`sign_lines`), via
    `sign_edit_centre_y`.
    Hanging signs are 125
    in both versions, and
    the 1.21.5 immediate-mode
    model draw versus
    1.21.6's batched render
    state is GPU plumbing
    (the Rust editor draws
    a 2D sprite either way).
  - `getDarkColor`: 1.21.6
    calls `ARGB.scaleRGB`
    (float), 1.21.5 scales
    each channel by 0.4 in
    doubles with alpha 0.
    The two agree for every
    channel value 0–255
    (checked exhaustively),
    so the Rust float form
    already matches 1.21.5
    exactly; the
    black-glowing special
    case matches too (the
    port's black text colour
    is 0), and the caller
    forces opaque alpha.
    No code change.
  - Verification: `cargo check -j 1
    -p horizon-client --lib --tests`
    0 errors / 0 warnings
    (type-checks the new test).
    The test-profile build was
    OOM-killed under 3-stream
    load, so the in-tree test
    runs in CI; its three
    assertions were verified
    by a standalone rustc
    run of the same function.
- **Item 123 — Container block
  entities: save/load and
  `removeComponentsFromTag`
  verified:**
  - The delta is the usual
    ValueInput/ValueOutput
    refactor with identical
    serialized form:
    `lock`, `CustomName`,
    `Items`. The Rust port
    reads and writes all
    three (`block_names`,
    `block_lock`, chest
    slots; the round-trip
    test in `block_items.rs`
    covers a locked barrel
    through save and reload).
  - `removeComponentsFromTag`
    (drop `CustomName`,
    `lock`, `Items` when the
    block entity is picked
    up as an item) is the
    `parts` table in
    `block_entity_item.rs`:
    every container type
    lists `NAME`, `LOCK`
    and `Part::Items(n)`,
    and `pick` removes
    each part's key from
    the carried data.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client --lib --tests`
    passed at item 122.
- **Item 124 — Loom screen
  verified against 1.21.5:**
  - The delta is the 1.21.6
    GuiRenderer refactor:
    RenderPipelines blits,
    a batched banner render
    state, and a flat
    pattern-sprite blit on
    the buttons, versus
    1.21.5's immediate-mode
    flag-model draws. The
    on-screen geometry is
    identical in both
    versions, verified from
    the flag model itself
    (box -10..10, 0..40,
    pivot y=-44): the
    1.21.5 result pose
    lands the flag at
    (141, 8, 20, 40), the
    1.21.6 rect, and the
    1.21.5 pattern-button
    pose lands it at
    (button+4, button+2,
    5, 10), the 1.21.6
    blit's rect. The Rust
    layout uses those
    rects, and the button's
    grey fill plus pattern
    blit reproduces the
    flat-lit flag face.
    (The 1.21.6 blit's UV
    is one texel off the
    flag face's — a quarter
    of a swatch pixel, not
    worth chasing.)
  - Scroller, grid, slots
    and error sprite all
    share coordinates in
    both versions, and the
    Rust layout matches.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client --lib --tests`
    passed at item 122.
- **Item 125 — Client packet
  listener verified against
  1.21.5:**
  - Command system: 1.21.6's
    NodeBuilder, its
    restricted nodes and the
    split suggestion
    providers (UNRESTRICTED
    vs RESTRICTED,
    `allowsRestrictedCommands`)
    are absent from the Rust
    port. The protocol layer
    decodes only 1.21.5's
    node flags (executable,
    redirect, suggestions;
    the restricted flag is
    1.21.6's), `from_packet`
    sets every network node
    to permission 0 ("the
    server sent only what
    the player may use"),
    and the suggestion
    permission is the
    player's own level
    (`owner_permission`),
    which is 1.21.5's
    `hasPermission`
    delegating to the local
    player.
  - `sendUnsignedCommand`
    already has the 1.21.5
    form: no confirm window,
    no parse or permission
    check; used by click
    commands and dialogs.
    No confirm-command code
    exists anywhere.
  - The mount onboard
    message already says
    `say_now` (1.21.5's
    name) with the overlay.
    Demo events have no
    surface: the game-event
    handler has no arm for
    event 5 and the port
    implements no demo mode,
    so the 1.21.6-only
    narration is moot.
  - Respawn: the port keeps
    the local player object
    across respawn and reads
    input live from the
    keyboard, so 1.21.6's
    `getLastSentInput` vs
    1.21.5's
    `isShiftKeyDown`
    distinction (the new
    player's initial input)
    has no surface.
  - Rider reposition: the
    inversion (1.21.5
    repositions the rider
    when the vehicle IS
    interpolating, 1.21.6
    when it is NOT) is a
    one-frame render detail;
    the port seats every
    rider at its vehicle's
    current position every
    tick (`position_riders`),
    so both forms collapse
    to the same outcome.
  - Chat signing: login
    resets the secure-chat
    session to default and
    rebuilds it only in
    online mode; no
    `Encoder.UNSIGNED`
    reset exists, matching
    1.21.5 (which has no
    such line).
  - Block-entity load reads
    the tag directly
    (`remote_block_entity_data`),
    no ValueInput layer;
    difficulty is the same
    two fields under
    renamed accessors;
    ServerData's renames
    are accessor-only.
  - One stale comment fixed:
    the reconfigure note in
    `remote.rs` cited
    1.21.6's
    `ClientWaypointManager`;
    1.21.5 has no waypoint
    manager at all.
  - Verification: `cargo check
    -j 1 -p horizon-client
    --lib` 0 errors / 0
    warnings.
- **Item 126 — Attribute
  instance save/load verified
  against 1.21.5:**
  - The delta is the usual
    API refactor: 1.21.6's
    `AttributeInstance.Packed`
    record and its codec
    versus 1.21.5's
    `save()`/`load(CompoundTag)`.
    The serialized form is
    identical in both
    versions — `id`, `base`
    (default 0.0),
    `modifiers` only when
    non-empty — and the Rust
    port already writes and
    reads exactly that form
    (`put_attributes`,
    `read_attribute_modifiers`,
    the bases load in
    `entity_nbt.rs`).
  - Only the stale 1.21.6
    names in comments needed
    changing: `AttributeMap.pack`
    → `AttributeMap.save`
    (12 sites across
    `entity_nbt.rs`,
    `entities.rs`,
    `motion.rs`, `game.rs`,
    `persist.rs`,
    `playerdata.rs`,
    `finalize.rs`,
    `finalize_load_tests.rs`,
    `ravager.rs`) and
    `AttributeInstance.apply`
    → `AttributeInstance.load`
    (`saved_attributes.rs`).
    The `Packed.LIST_CODEC`
    citation in `motion.rs`
    is now `AttributeMap.save`'s
    ListTag form.
  - The Rust-side
    `pack()`/`apply()` API
    in `horizon-world`'s
    attributes module is
    left alone: it is an
    internal Rust API, not
    the serialized form,
    and renaming it would be
    churn with no observable
    effect.
  - Pre-existing edge, same
    in both versions: a
    saved entry with no
    `base` is skipped by the
    Rust load where vanilla
    sets 0.0 — reachable
    only from hand-edited
    NBT, since `save`
    always writes `base`.
  - Verification: `cargo check
    -j 1 -p horizon-client
    --lib` 0 errors / 0
    warnings.
- **Item 127 — Cat save and
  variant selection verified
  against 1.21.5:**
  - Same pattern as item 112
    (chicken/pig/fish/salmon):
    save/load is the usual
    ValueInput/ValueOutput →
    CompoundTag refactor with
    identical fields —
    variant via
    `VariantUtils.writeVariant`
    / `readVariant`, and
    `CollarColor` via
    `DyeColor.LEGACY_ID_CODEC`
    defaulting to red. The
    Rust port writes the
    variant name (`Cat` is in
    `saves_variant_name`) and
    the collar colour as a
    byte, and reads both back
    with red as the default
    (`tame::RED` = 14).
  - Variant selection is
    already the 1.21.5
    per-type form:
    `finalize` calls the cat
    registry's `select`
    (`VariantRegistry::select`,
    highest-priority entries
    whose condition holds,
    one at random) — the
    shape of 1.21.5's
    `CatVariants.selectVariantToSpawn`,
    not 1.21.6's generic
    `VariantUtils.selectVariantToSpawn`.
    The comments already cite
    the 1.21.5 form.
  - Pre-existing gap carried
    from item 69: swamp-hut
    cats are always black in
    vanilla, uniform random
    in the Rust port.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client --lib`
    passed at item 126.
- **Item 128 — Crafter screen
  verified against 1.21.5:**
  - The RenderPipelines →
    RenderType hunks are GPU
    plumbing with no Rust
    surface; the disabled-
    slot sprite, the
    powered/unpowered
    redstone indicator and
    the background are drawn
    by the verified
    `screen_layout::crafter`
    layout.
  - The tooltip half already
    matches 1.21.5:
    `crafter_tooltip` shows
    `gui.togglable_slot`
    over an empty, enabled
    grid slot with nothing
    carried and the player
    not a spectator — the
    exact 1.21.5 condition
    — and `draw_slot_tooltip`
    draws it in the same
    frame, which is 1.21.5's
    immediate `renderTooltip`,
    not 1.21.6's deferred
    `setTooltipForNextFrame`.
    (The deferred form would
    only cost one frame of
    latency anyway, and the
    tooltip's empty-carried
    condition rules out any
    z-order difference.)
  - `CrafterMenu`'s
    `level()` → `serverLevel()`
    is an accessor rename
    returning the same
    ServerLevel.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client --lib`
    passed at item 126.
- **Item 129 — Drowned
  navigation back to the
  1.21.5 dual form:**
  - 1.21.6 gave the drowned
    one `AmphibiousPathNavigation`;
    1.21.5 keeps two
    navigations and swaps
    them in `updateSwimming`:
    water-bound while
    swimming, ground
    otherwise, with
    `DrownedGoToWaterGoal.start`
    also selecting the ground
    one. The port had the
    1.21.6 form (a fixed
    `NavKind::Amphibious`).
  - CODE: the drowned now
    starts on `NavKind::Ground`
    (1.21.5's `createNavigation`
    default — Drowned does
    not override it),
    `drowned_update_swimming`
    swaps the kind with the
    swimming flag
    (`Water { breach: false }`
    while swimming), and the
    swap drops the current
    path, as vanilla's fresh
    navigation object starts
    pathless. The go-to-water
    goal's `start` selects
    the ground navigation
    before moving, as vanilla
    does. The `Amphibious`
    doc list in `nav.rs` no
    longer names the drowned.
  - The swap is observable
    through the navigation
    itself: the evaluator
    (Swim vs Walk),
    `can_update_path` (in
    liquid vs on ground),
    and the path origin
    (mid-height vs surface)
    all follow the kind, and
    `can_navigate_ground`
    now answers dynamically
    as vanilla's
    `hasGroundPathNavigation`
    does.
  - Axolotl, Turtle and Frog
    keep `AmphibiousPathNavigation`
    in 1.21.5 (verified in
    the 1.21.5 decomp), so
    only the drowned changes.
  - Test added:
    `a_drowned_swaps_navigation_with_its_swimming`
    (Ground at rest, Water
    under water while it
    wants to swim, Ground
    again out of water, path
    dropped on each swap).
  - Verification: `cargo check
    -j 1 -p horizon-client
    --lib --tests` 0 errors /
    0 warnings; test 1 passed.
- **Item 130 — Minecarts
  verified against 1.21.5:**
  - `recreateFromPacket` is
    1.21.6-only (it seeds the
    new behaviour's lerp
    motion from the packet
    velocity); 1.21.5 has no
    such override. The Rust
    spawn path already
    matches 1.21.5: AddEntity
    inserts a fresh
    `CartLerp` (zero movement,
    empty steps) and sets the
    cart's velocity directly;
    the lerp is fed only by
    later move packets.
  - `applyEffectsFromBlocks`:
    1.21.6's non-experimental
    branch clears the
    recorded movements after
    applying the single-
    segment effects, 1.21.5's
    does not. No Rust surface:
    the port has no
    movement-recording system
    (`movementThisTick` /
    `finalMovementsThisTick`
    exist nowhere in the
    tree); the cart applies
    its block effects directly
    over the step
    (`cart_inside_blocks`),
    which is the 2-arg form
    both versions call.
  - Save/load is the usual
    ValueInput/ValueOutput →
    CompoundTag refactor with
    identical fields, all
    read and written by the
    port: `DisplayState`,
    `DisplayOffset` (default
    when absent),
    `FlippedRotation`,
    `HasTicked`, hopper
    `Enabled` (default true),
    TNT `fuse` (default -1).
  - Pre-existing gap, same in
    both versions: the port
    does not carry TNT carts'
    `explosion_power` (it
    explodes at the fixed
    4.0 base); the field
    exists identically in
    1.21.5 and 1.21.6, so it
    is not this delta.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client --lib --tests`
    passed at item 129.
- **Item 131 — Locator bar
  game rule removed; sweep
  and collisions verified:**
  - CODE: `RULE_LOCATOR_BAR`
    does not exist in 1.21.5.
    Removed `locator_bar`
    from the rule registry
    (`gamerules.rs`, which
    feeds `/gamerule` and its
    suggestions) and from
    the new-world defaults
    table (`save.rs`
    `GAME_RULES`, 58 → 57
    entries), so a new
    world's `game_rules.dat`
    no longer carries a key
    vanilla 1.21.5 would
    reject. No code read the
    rule. The datafix entries
    stay: they are version-
    upgrade history (item
    108's decision), not the
    live bar.
  - `forEachBlockIntersectedBetween`:
    1.21.6's visitor returns
    boolean and stops the
    traversal when the entity
    dies mid-sweep; 1.21.5's
    is void and visits on,
    applying nothing after
    death. The observable
    outcome is identical —
    no effects after death in
    either version — so the
    Rust sweep keeps its
    bool-returning visitor,
    whose early exit serves
    hit detection (arrows,
    projectiles, falling
    blocks), mirroring
    vanilla's raycast, not
    the 1.21.6 visitor.
  - The Ghast half was already
    the 1.21.5 form:
    `ghast_move_control`
    steps the box one block
    at a time with
    `no_collision` and
    accelerates by a fixed
    0.1, with comments citing
    the 1.21.6 sweep form it
    is not.
  - `getPreMoveCollisions`
    (1.21.6-only) has no Rust
    surface: the LAN move
    check already uses
    `getCollisions` via
    `BlockCollisionScan`,
    with a comment noting the
    1.21.6 rename. The
    `getBlockCollisions`
    refactor (context built
    inside vs outside) is
    behaviour-identical.
  - `GameType.STREAM_CODEC`
    (1.21.6-only) has no
    surface: the protocol
    writes the mode as a
    plain varint. The
    `updateRules` copy-vs-
    share distinction has
    none either: Rust rule
    values are `Copy`, so
    there are no shared
    mutable value objects.
  - Verification: `cargo check
    -j 1 -p horizon-client-core
    -p horizon-client --lib`
    0 errors / 0 warnings.
- **Item 132 — Window GUI
  scale verified against
  1.21.5:**
  - The delta is only the
    storage type of
    `Window.guiScale`: double
    in 1.21.5, int in 1.21.6.
    It is unobservable:
    `calculateScale` returns
    an int in both versions
    and `setGuiScale` is only
    ever called with that
    int, so the stored value
    is always whole. The
    scaled-size computation
    (ceil of framebuffer /
    scale) is identical in
    both versions.
  - The Rust port matches:
    `calculate_scale`
    (`horizon-client-input`)
    is the vanilla loop
    verbatim, including the
    framebuffer bounds and
    the unicode even-bump;
    `gui_scaled_size` is the
    ceil division; `HudBatch`
    stores the scale as
    `f32` (the 1.21.5
    double-storage analogue)
    with ceil-divided width
    and height.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client-core
    -p horizon-client --lib`
    passed at item 131.
- **Item 133 — Slime save/load
  verified against 1.21.5:**
  - The usual ValueInput/
    ValueOutput → CompoundTag
    refactor with identical
    fields: `Size` (saved as
    size-1, read as +1,
    default 0 → size 1) and
    `wasOnGround` (default
    false). The Rust port
    writes and reads exactly
    that form for slimes and
    magma cubes, and reads
    the size first, as
    vanilla does (the size's
    attribute bases — max
    health size², speed
    0.2+0.1·size, attack
    size, magma armour
    3·size — are set by
    `set_size` before the
    saved health is applied).
  - No stale 1.21.6 names in
    the slime comments.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client-core
    -p horizon-client --lib`
    passed at item 131.
- **Item 134 — Advancement
  load and /defaultgamemode
  verified against 1.21.5:**
  - The JSON parser change
    is a refactor with the
    same behaviour: 1.21.5
    parses with a non-lenient
    `JsonReader`, 1.21.6 with
    `StrictJsonParser` —
    both strict. The Rust
    port loads with
    `serde_json::from_str`,
    strict by default, and
    falls back to empty
    progress on failure, as
    vanilla does (it logs
    and continues).
  - `/defaultgamemode` is at
    level 2 (`GAMEMASTERS`)
    in the Rust tree, matching
    1.21.5's `hasPermission(2)`;
    the `Commands.hasPermission`
    → lambda change is the
    same pure refactor as
    item 110. The `level()` →
    `serverLevel()` change is
    an accessor rename.
  - The announcement gate
    matches in structure:
    display `announce` and
    the advancement-messages
    rule both true.
  - Pre-existing base drift,
    not this delta: the Rust
    rule is named
    `show_advancement_messages`
    (the 26.3 name) where
    both 1.21.5 and 1.21.6
    call it
    `announceAdvancements`.
    The port reads its own
    name consistently, so
    behaviour is self-
    consistent; only cross-
    compat with vanilla
    saves and `/gamerule`
    spellings is affected.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client-core
    -p horizon-client --lib`
    passed at item 131.
- **Item 135 — Chunk block-
  entity add verified
  against 1.21.5:**
  - `Level.onBlockEntityAdded`
    is 1.21.6-only: a no-op
    on the server, and on the
    client it puts off-screen
    block entities into the
    global render list (the
    1.21.6 mechanism behind
    item 113's section-list
    split). The Rust port has
    no such hook and no
    separate global list —
    off-screen block entities
    stay in the section list
    and are exempted from
    frustum culling at render
    time, which is the 1.21.5
    form item 113 verified.
  - The block-entity load
    half is the usual
    refactor: 1.21.6 wraps it
    in a `ProblemReporter`
    collector and loads via
    `TagValueInput`, 1.21.5
    loads the tag directly.
    The Rust port loads block
    entities from NBT
    directly; no
    `ProblemReporter` exists
    in the tree.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client-core
    -p horizon-client --lib`
    passed at item 131.
- **Item 136 — NBT list
  count verified against
  1.21.5:**
  - 1.21.6's `readListCount`
    rejects a negative list
    length with an
    `NbtFormatException`;
    1.21.5 reads the int
    unchecked. The observable
    outcome is the same
    either way: 1.21.5's
    `loadList` then builds
    `new ArrayList(count)`,
    which throws
    `IllegalArgumentException`
    for a negative capacity,
    so the load fails in both
    versions — only the
    exception type differs,
    and the Rust port has its
    own error type anyway.
    The Rust reader's shared
    `count()` helper rejects
    negative collection
    lengths ("negative
    collection length"), in
    both the normal and the
    streaming list paths, so
    it matches 1.21.5's
    observable behaviour.
    (The same holds for byte
    / int / long arrays:
    vanilla throws
    `NegativeArraySizeException`,
    the port errors.)
  - `KnownPack`'s `id()` →
    `getId()` is an accessor
    rename; the Rust pack
    system has no `KnownPack`
    type.
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client-core
    -p horizon-client --lib`
    passed at item 131.
- **Item 137 — Sniffer model
  and leash verified against
  1.21.5:**
  - The model delta is a pure
    refactor: 1.21.6 bakes
    the eight animations into
    `KeyframeAnimation`
    fields at construction,
    1.21.5 looks the
    definitions up per call.
    Same animations, same
    parameters, same order.
    The Rust port already has
    the 1.21.5 shape: `anim`
    looks each animation up
    by name from the map
    every frame, applies the
    walk (sniff-search when
    searching, else walk, at
    9.0 / 100.0) and the five
    state animations in
    vanilla's order (dig,
    long sniff, stand up,
    happy, sniff-sniff).
  - `supportQuadLeash` /
    `getQuadLeashOffsets`
    are 1.21.6-only and have
    no Rust surface: no
    quad-leash code exists
    anywhere in the tree —
    the leash system is
    already the 1.21.5
    single-leash form (items
    69, 120).
  - The animation
    definitions themselves
    are unchanged between
    the versions (no
    `SnifferAnimation` diff).
  - Verification: no Rust
    change; the tree state is
    the one `cargo check -j 1
    -p horizon-client-core
    -p horizon-client --lib`
    passed at item 131.
- **Item 138 — Item model
  wrappers verified against
  1.21.5:**
  - `ClientItem.Properties`:
    the `oversizedInGui`
    half was already removed
    by item 121; the Rust
    `ClientItem` keeps only
    `model`,
    `hand_animation_on_swap`
    and `swap_animation_scale`
    (the last is a
    later-version property
    kept for the hand-swap
    animation, default 1.0).
  - Special-model extents
    are already the 1.21.5
    form: `special_extents`
    returns the fixed
    unit-cube corners
    (`SpecialModelWrapper.
    EXTENTS`) for every
    special model, in
    vanilla's order — not
    the 1.21.6 per-renderer
    `getExtents`. The stale
    module doc in
    `special.rs` still
    described the 1.21.6
    form; fixed to cite the
    fixed extents.
  - The four
    `appendModelIdentityElement`
    removals (special,
    composite, conditional,
    select) have no surface:
    the Rust port has no
    model-identity concept
    (item 121).
  - Foil `setAnimated`:
    1.21.6's special-model
    wrapper marks a foiled
    stack animated, 1.21.5's
    doesn't. The Rust port
    sets `animated` for any
    foiled layer, but the
    only consumer is the GUI
    draw cache, whose key
    builder independently
    excludes foiled layers
    and every special-model
    layer — so the flag is
    unobservable for special
    models and no change is
    needed.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib`
    0 errors, 0 warnings
    (2m 58s).
- **Item 139 — Integrated
  player save verified
  against 1.21.5:**
  - The save half is the
    usual `ValueOutput`
    refactor: 1.21.6 saves
    the owner through a
    `ProblemReporter`
    collector and
    `TagValueOutput`,
    1.21.5 saves into a
    plain `CompoundTag`.
    The serialized form is
    identical (item 91
    verified the player
    save fields). The Rust
    port writes player NBT
    directly (`write_player`
    in `save.rs`, via the
    autosave path); no
    `ProblemReporter` or
    `TagValueOutput` layer
    exists in the tree.
  - `SharedConstants.
    getCurrentVersion().
    name()` → `getName()`
    is an accessor rename
    in a log line.
  - Verification: no Rust
    change; the tree state
    is the one `cargo check
    -j 1 -p horizon-client
    --lib` passed at item
    138.
- **Item 140 — Look control
  verified against 1.21.5:**
  - 1.21.5's
    `setLookAt(Entity)` aims
    at a living entity's eye
    height and at a
    non-living entity's
    bounding-box centre;
    1.21.6 always uses the
    eye height. The Rust
    `LookControl` takes a
    raw point (no entity
    overload), and every
    call site that targets
    an entity targets a
    living one — combat
    targets, players and
    followed mobs via
    `target_info` /
    `eye_y`, the value
    `getWantedY` returns
    for living entities in
    both versions. The
    brain look-target sinks
    resolve through
    `Tracker::position`,
    which also goes through
    `target_info`; non-living
    entities are not in the
    mob world's target
    table, so the 1.21.5
    bounding-box-centre
    branch is unreachable
    in the Rust port. The
    remaining call sites
    aim at block positions
    or computed points.
  - Verification: no Rust
    change; the tree state
    is the one `cargo check
    -j 1 -p horizon-client
    --lib` passed at item
    138.
- **Item 141 — Recipe
  display lookup verified
  against 1.21.5, with one
  deliberate deviation:**
  - `getRecipeFromDisplay`:
    1.21.6 bounds-checks the
    index and returns null
    out of range; 1.21.5
    indexes `allDisplays`
    directly, so an
    out-of-range display id
    throws
    `IndexOutOfBoundsException`
    out of the packet
    handler — a server
    crash on a malformed
    guest packet (the 1.21.6
    bounds check is the fix
    for it). The Rust port
    keeps the safe form:
    `recipe_of_display`
    returns `Option` and
    both server handlers
    (`lan_place_recipe`,
    the seen-recipe handler
    in `hub.rs`) no-op on
    `None`, matching the
    behaviour the 1.21.5
    handlers' own null
    checks were written for.
    The port does not
    replicate the crash.
  - Advancement validation:
    the log-line change is
    same-behaviour (log
    only when problems
    exist), and the Rust
    port has no advancement
    criterion validator at
    all (items 71, 82, 97).
  - Verification: no Rust
    change; the tree state
    is the one `cargo check
    -j 1 -p horizon-client
    --lib` passed at item
    138.
- **Item 142 — Command
  suggestions and /recipe
  verified against 1.21.5:**
  - The suggestion delta is
    plumbing: 1.21.6 routes
    registry suggestions
    through a static
    `listSuggestions`
    helper and a
    `HolderLookup`, 1.21.5
    suggests from the
    argument's own registry
    lookup directly. The
    suggested content is
    identical — tags with a
    `#` prefix, then element
    ids — and the Rust
    engine already has the
    1.21.5 shape: it always
    suggests from the local
    registry tables, tags
    first with `#` for
    `ResOrTag` /
    `ResOrTagKey`, element
    ids only for
    `ResourceSelector`. The
    non-provider-source
    fallback differs
    between the versions,
    but every reachable
    source (local player,
    `CommandSourceStack`)
    is a provider.
  - `/recipe` is at level 2
    (`GAMEMASTERS`) in the
    tree; the
    `Commands.hasPermission`
    → lambda change is the
    same pure refactor as
    items 110 and 134.
  - `HexColorArgument` is
    1.21.6-only (used only
    by `/waypoint modify ...
    color hex`), but its
    removal belongs with
    the `/waypoint` command
    items (36/52/145): the
    Rust `ArgType::HexColor`
    is still referenced by
    the waypoint tree in
    `tree_extra.rs`, and
    removing the type
    before the command
    would break the tree.
    Left in place, noted
    here.
  - Verification: no Rust
    change; the tree state
    is the one `cargo check
    -j 1 -p horizon-client
    --lib` passed at item
    138.
- **Item 143 — Block sweep
  visitor verified against
  1.21.5 (decided at item
  131):**
  - `forEachBlockIntersected
    Between` takes a
    boolean visitor in
    1.21.6 and a void one
    in 1.21.5. The only
    caller that cares is
    `Entity.checkInsideBlocks`:
    1.21.6's visitor
    returns false to stop
    the sweep when the
    entity is dead,
    1.21.5's keeps visiting
    but applies nothing
    (every effect is behind
    an `isAlive()` guard,
    including the visited-
    set add), so the extra
    visits are
    unobservable. The Rust
    sweep keeps its bool
    visitor: its early exit
    serves hit detection
    (arrows, projectiles,
    falling blocks stop at
    the first hit),
    mirroring vanilla's
    raycast — the decision
    recorded at item 131.
    The ghast caller is the
    1.21.6 sweep form of
    `canReach`; the Rust
    ghast already uses the
    1.21.5 stepped-box form
    (item 131).
  - Verification: no Rust
    change; the tree state
    is the one `cargo check
    -j 1 -p horizon-client
    --lib` passed at item
    138.
- **Item 144 — Leash system,
  happy ghast, collision
  predicate verified
  against 1.21.5:**
  - The leash system is
    already the full 1.21.5
    form (`mobs/leash.rs`,
    items 69/90/120):
    snap at 10.0 with no
    sound, elastic at 6.0
    with the legacy
    `copySign(unit^2 * 0.4)`
    pull, feet-to-feet
    `distanceTo` narrowed
    to float, the
    `handleLeashAtDistance`
    gate (sitting wolf /
    parrot / cat not pulled,
    lead drops past 10;
    home set to the
    holder's block, radius
    5), `checkSlowFallDistance`
    after the elastic pull,
    and close-range
    behaviour. Attach is
    the 1.21.5
    `canHaveALeashAttachedToIt`
    form (`can_be_leashed
    && alive && !leashed`,
    no distance check). No
    wrench, angular
    momentum, quad-leash or
    spring code exists
    anywhere in the tree.
  - Happy ghast: no Rust
    surface at all — no
    species, no spawn
    placement, no entity.
    That also settles the
    `canBeCollidedWith`
    parameter: only the
    happy ghast's override
    uses it (shulker and
    boat ignore it), and
    the Rust tree has no
    `can_be_collided_with`
    predicate at all.
    (Name-keyed dried-ghast
    block code in
    `animate_tick` /
    `level` is inert: the
    block is not in the
    1.21.5 tables.)
  - `EntityAttachments.
    getAverage` is
    1.21.6-only and has no
    Rust surface (no
    attachment averaging).
  - `RAVAGER_ROARED` (69)
    not existing in 1.21.5
    is item 81's ground:
    the server no longer
    broadcasts event 69 and
    the client roar runs on
    the client tick.
  - LightningBolt / Marker
    save and load nothing
    in either version; the
    `ValueInput` /
    `ValueOutput` change
    is the usual refactor.
  - Verification: no Rust
    change; the tree state
    is the one `cargo check
    -j 1 -p horizon-client
    --lib` passed at item
    138.
- **Item 145 — `/waypoint`
  and `/dialog` removed —
  CODE:**
  - Both commands arrived
    in 1.21.6 and are gone
    from the tree: the
    command trees in
    `tree_extra.rs`, the
    `waypoint.*` /
    `dialog.*` dispatch in
    `commands.rs`, the
    executors in
    `commands/extra.rs`
    (`cmd_waypoint`,
    `cmd_dialog`, the
    `WaypointArgument`
    equivalent
    `waypoint_arg`, the
    server waypoint
    manager
    `waypoint_transmitters`
    / `tick_waypoints` /
    `transmit_range`, the
    `WaypointIcon` state
    and the `waypoints` /
    `waypoint_icons` /
    `waypoint_dirty`
    fields), the dialog
    delivery helpers
    (`show_dialog_to`,
    `send_dialog`,
    `clear_dialog_to`,
    `dialog_holder`), the
    `/attribute` hook that
    marked
    `waypoint_transmit_range`
    changes dirty, and the
    five
    `commands.waypoint.*`
    lang keys. The dialog
    screen functions that
    lost their only callers
    went too
    (`show_play_dialog` —
    the handler for the
    1.21.6-only
    `ClientboundShowDialogPacket`
    — and `clear_dialog`).
  - `ArgType::HexColor`
    went with the waypoint
    tree, its only use
    (item 142): the variant,
    its parse and suggest
    arms, the
    `minecraft:hex_color`
    network decode, and
    the command-parse
    fixture mapping.
  - What stays, for its
    owning items: the
    `waypoint_transmit_range`
    / `waypoint_receive_range`
    attributes (item 225
    owns the registry,
    260 the player hooks,
    264 the invisibility
    modifier), the
    attribute-update
    waypoint events, and
    the dialog UI system
    (`dialogs.rs`, items
    175/219/222/258). The
    waypoint and dialog
    packets were already
    absent (`play.rs`
    notes 1.21.5 has
    neither).
  - The four permission
    refactors are the same
    pure form as items 110
    / 134 / 142; the levels
    already match: `/jfr`
    4, `/kick` 3,
    `/publish` 4,
    `/spectate` 2.
  - Verification: `cargo
    check -j 1 -p
    horizon-command -p
    horizon-client --lib`
    0 errors, 0 warnings
    (1m 25s); `cargo check
    -j 1 -p
    horizon-client-lib
    --example client_probe`
    0 errors, 0 warnings
    (3m 38s).
- **Item 146 — Warden model
  verified against 1.21.5:**
  - The same pure refactor
    as item 137's sniffer:
    1.21.6 bakes the six
    animations into
    `KeyframeAnimation`
    fields at construction,
    1.21.5 looks the
    definitions up per call.
    Same animations, same
    parameters, same order.
    The Rust port already
    has the 1.21.5 shape:
    `warden::anim` looks
    each animation up by
    name from the map every
    frame and applies all
    six in vanilla's order
    (attack, sonic boom,
    dig, emerge, roar,
    sniff), after the
    inline walk, idle-pose
    and tendril poses.
  - The animation
    definitions themselves
    are unchanged between
    the versions (no
    `WardenAnimation`
    diff).
  - Verification: no Rust
    change; the tree state
    is the one `cargo check
    -j 1 -p horizon-command
    -p horizon-client --lib`
    passed at item 145.
- **Item 147 — StreamCodec
  composite verified
  against 1.21.5:**
  - The 11-arity
    `StreamCodec.composite`
    overload is 1.21.6-only;
    1.21.5 tops out at the
    smaller arities. It has
    no Rust surface: the
    protocol crate has no
    composite combinator at
    all — every packet codec
    is hand-written field
    by field, so a Java
    combinator's arity
    limit cannot bite. No
    1.21.6 changed class
    even calls the 11-arity
    form (only
    `StreamCodec` itself
    mentions `Function11`),
    so no 1.21.5 packet
    codec is shaped by its
    absence.
  - Verification: no Rust
    change; the tree state
    is the one `cargo check
    -j 1 -p horizon-command
    -p horizon-client --lib`
    passed at item 145.
- **Item 148 — Resource-pack
  prompt parsing verified
  against 1.21.5:**
  - 1.21.6 parses the
    prompt with
    `StrictJsonParser` +
    the component CODEC
    (logging the codec
    error); 1.21.5 uses
    `Component.Serializer.
    fromJson`, which is
    strict `JsonParser.
    parseString` plus the
    old `deserialize` —
    not the lenient variant
    (`fromJsonLenient` is a
    separate method the
    prompt path never
    calls). The Rust port
    matches the 1.21.5
    form: `parse_prompt`
    runs the strict
    `json::parse` and a
    component-shape check
    (string / number /
    bool, non-empty array
    of components, or an
    object with a content
    key — the shapes
    `deserialize`
    accepts), warning and
    returning no prompt on
    either failure, as
    1.21.5's catch does.
  - Verification: no Rust
    change; the tree state
    is the one `cargo check
    -j 1 -p horizon-command
    -p horizon-client --lib`
    passed at item 145.
- **Item 149 — Raid trigger
  verified against 1.21.5:**
  - `ServerPlayer.level()`
    → `serverLevel()` in
    `createOrExtendRaid` is
    a pure rename: on a
    server player both
    return the same
    `ServerLevel`, and the
    disable-raids rule
    check reads the same
    object either way. No
    behaviour change, no
    Rust surface (the
    port's raid code in
    `game/raids.rs` reads
    the rule from its own
    level state; the port
    names the rule `raids`,
    inverted from vanilla's
    `disableRaids`, a
    version-neutral
    convention).
  - Verification: no Rust
    change; the tree state
    is the one `cargo check
    -j 1 -p horizon-command
    -p horizon-client --lib`
    passed at item 145.
- **Item 150 — Beehive
  ignored tags verified
  against 1.21.5:**
  - `IGNORED_BEE_TAGS` is
    already the 1.21.5 form
    in `beehive.rs`, all 29
    entries in vanilla's
    order: `ArmorItems` /
    `HandItems` (not
    1.21.6's `equipment`)
    and `SleepingX` /
    `SleepingY` /
    `SleepingZ` (not
    `sleeping_pos`), as
    the doc comment notes.
    The occupant timing
    matches too (nectar →
    2400 ticks, else 600),
    pinned by a test. The
    load / save /
    `removeComponentsFromTag`
    halves are the usual
    `ValueInput` /
    `ValueOutput` →
    `CompoundTag` refactor
    with an identical
    serialized form (`bees`
    list, nullable
    `flower_pos`).
  - Verification: no Rust
    change; the tree state
    is the one `cargo check
    -j 1 -p horizon-command
    -p horizon-client --lib`
    passed at item 145.
- **Item 151 — Explosion
  water check verified
  against 1.21.5:**
  - 1.21.5's
    `shouldAffectBlocklikeEntities`
    spares blocklike
    entities when the blast
    source is in water;
    1.21.6 dropped that
    term (moving the water
    check into
    `BlockAttachedEntity.
    ignoreExplosion`
    alone, so it no longer
    spares items or armour
    stands). The Rust port
    already has the 1.21.5
    form in both places
    the predicate feeds:
    `items_ignore_explosion`
    and
    `blast_spares_decorations`
    both return true on
    `source_in_water`, with
    doc comments recording
    the version difference
    and a test pinning it
    ("a blast whose source
    is in water never
    affects blocklike
    entities, griefing or
    no"). The wind-charge
    term is vacuous on the
    Rust path: no wind
    charge goes through
    `server_explode`.
  - Verification: no Rust
    change; the tree state
    is the one `cargo check
    -j 1 -p horizon-command
    -p horizon-client --lib`
    passed at item 145.
- **Item 152 — Anvil take
  verified against 1.21.5:**
  - 1.21.6's `onTake` calls
    the text filter's
    `processStreamMessage`
    on the new item name;
    1.21.5 has no such
    call. The Rust take
    path (`anvil_used`)
    makes no filter call
    either — it does the
    12% damage roll, the
    damage stages and the
    level events (1029 on
    break, 1030 otherwise),
    matching 1.21.5's
    `onTake`. (The port's
    rename-time filter in
    `lan_station_rename`
    is a Horizon LAN
    moderation feature,
    not vanilla behaviour:
    vanilla's
    `handleRenameItem` is
    identical in both
    versions — direct
    `setItemName`, no
    filter — so it is
    version-neutral and
    out of scope here.)
  - Verification: no Rust
    change; the tree state
    is the one `cargo check
    -j 1 -p horizon-command
    -p horizon-client --lib`
    passed at item 145.
- **Item 153 — Limited
  crafting verified against
  1.21.5:**
  - `ServerPlayer.level()`
    → `serverLevel()` in
    `setRecipeUsed` is the
    same pure rename as
    item 149: both return
    the same `ServerLevel`.
    The gate itself is
    implemented in the
    Rust crafting menu:
    with limited crafting
    on, only special
    recipes or recipes the
    player's book knows
    produce a result.
  - Verification: no Rust
    change; the tree state
    is the one `cargo check
    -j 1 -p horizon-command
    -p horizon-client --lib`
    passed at item 145.
- **Item 154 — Elder
  guardian home verified
  against 1.21.5:**
  - `hasHome()` /
    `setHomeTo(pos, 16)`
    (1.21.6) vs
    `hasRestriction()` /
    `restrictTo(pos, 16)`
    (1.21.5) is a rename
    of the restriction API
    with identical
    behaviour. The Rust
    `elder_server_ai_step`
    already does the 1.21.5
    form: the minute check
    `(age + id) % 1200 ==
    0` with mining fatigue
    III in 50 blocks, then
    a home at the block
    position, radius 16,
    when it has none
    (pinned by the
    `an_elder_guardian_homes_where_it_stands`
    test). One stale
    1.21.6 method name in
    that test's comment
    fixed (`setHomeTo` →
    `restrictTo`); the
    Rust `ai.home` field
    name stays (internal
    naming only, item 74).
  - Verification: comment
    only; the tree state is
    the one `cargo check
    -j 1 -p horizon-command
    -p horizon-client --lib`
    passed at item 145.
- **Item 155 — Win screen
  narration removed (1.21.6
  only):**
  - 1.21.6 narrates the
    poem and credits
    (`narratorComponents`,
    `getNarrationMessage`
    joining them);
    1.21.5 has no such
    override and the
    screen's title is
    `GameNarrator.NO_TITLE`,
    so it narrates
    nothing. Removed the
    narration build from
    `win_screen.rs`: the
    `narration` field, the
    `narrated` collection
    in `new`,
    `add_poem_file` and
    `add_credits_file`,
    and the third argument
    of `add_credits_line`
    (back to 1.21.5's
    two-argument form);
    `screen_info` now
    carries an empty
    narration. The lines
    themselves — poem
    wrapping at 256 px,
    credits headings and
    names, scroll — are
    untouched. The two
    tests that pinned the
    1.21.6 narration now
    pin the 1.21.5 form
    (empty title, id
    still `win`). The
    render-plumbing
    differences
    (`pushMatrix` /
    `RenderPipelines`
    vs `pushPose` /
    `RenderType`) have no
    Rust surface: the
    poem's End-portal
    background and the
    vignette are drawn by
    the port's own
    renderer already.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib
    --tests` 0/0 (4m 07s),
    which type-checks the
    reworked tests. The
    test-profile build
    itself was OOM-killed
    under 3-stream load
    (item 122's trap), so
    the tests were not run
    locally; they run in
    CI. The reworked
    assertions are direct
    consequences of the
    change (`screen_info`
    now carries `""`).
- **Item 156 — Player
  inventory body/saddle
  slots removed; abilities
  save verified:**
  - `SLOT_BODY_ARMOR`
    (41) and `SLOT_SADDLE`
    (42) are 1.21.6-only:
    1.21.5's
    `EQUIPMENT_SLOT_MAPPING`
    ends at OFFHAND (40),
    so the container size
    is 41, `getItem(41/42)`
    returns empty and
    `setItem(41/42)` is a
    no-op. Removed the
    41/42 arms from
    `EquipmentSlot::inventory_slot`
    and
    `from_inventory_slot`
    (`player_inventory.rs`);
    `Body` and `Saddle`
    stay entity equipment
    slots (horses, wolves)
    and keep their
    `/item` `armor.body` /
    `saddle` names. The
    parity fixtures and
    drivers now pin a
    five-entry mapping
    order (`[EquipmentSlot;
    5]`, `range(5)`) in
    place of seven; the
    `0..43` snapshot range
    and the slot-42 edge
    cases stay, and now
    test the unmapped-slot
    no-op on both sides
    (the oracles clear the
    map and put only the
    five ordered entries).
    Inventory and
    abilities save/load
    were already the
    1.21.5 `CompoundTag`
    / `ListTag` form
    (`Slot` byte + item,
    `abilities` compound
    with the same seven
    keys and defaults);
    the 1.21.6
    `ValueInput` /
    `ValueOutput` and
    `Abilities.Packed`
    APIs have no Rust
    surface.
  - Verification: `cargo
    check -j 1 -p
    horizon-world --lib
    --tests --examples`
    0/0 (2m 17s).
- **Item 157 — Model
  animation helpers
  verified against
  1.21.5:**
  - 1.21.5's base `Model`
    carries `animate`,
    `animateWalk` and
    `applyStatic` (via
    `KeyframeAnimations.animate`
    on the definition);
    1.21.6 deleted them
    when it moved to baked
    `KeyframeAnimation`
    fields. The Rust port
    already has the 1.21.5
    shape: `Animation::apply`
    and `apply_walk` in
    `keyframes.rs`
    evaluate the
    definition per call,
    and `apply_walk` is
    `animateWalk` verbatim
    (time `pos * 50 *
    speed_factor`, scale
    `min(speed *
    scale_factor, 1)`).
    `applyStatic` (time 0)
    and
    `getAnyDescendantWithName`
    have no Rust surface
    (nothing applies a
    static pose; parts are
    looked up by name in
    the `Poses` map).
    Five doc comments
    that cited the 1.21.6
    baked class fixed to
    cite `Model.animate` /
    `Model.animateWalk` /
    `KeyframeAnimations.animate`
    instead.
  - Verification: comments
    only; the tree state
    is the one `cargo
    check -j 1 -p
    horizon-client --lib
    --tests` passed at
    item 155.
- **Item 158 — Texture
  filter/blur plumbing
  verified against
  1.21.5:**
  - `AbstractTexture`'s
    texture view and
    `setUseMipmaps` are
    1.21.6 GPU plumbing
    with no Rust surface
    (the port manages
    samplers and mipmap
    levels in its own
    wgpu texture code,
    driven by the Mipmap
    Levels option).
    1.21.5's
    `setFilter(TriState,
    ...)` / `defaultBlur`
    form is behaviourally
    inert: both TriState
    callers in the 1.21.5
    client (SkyRenderer,
    WorldBorderRenderer)
    pass explicit FALSE,
    never DEFAULT, so the
    default never decides
    anything.
    `OverlayTexture`'s
    1.21.5 `setFilter(false,
    false)` has no surface
    either — the port has
    no overlay texture,
    the hurt red and white
    flash are WGSL
    constants
    (`overlay.rs`).
    `TextureAtlasSprite.isAnimated()`
    (1.21.6-only) has no
    surface: the port
    derives animation from
    a sprite's frames
    list, with no
    `is_animated` field.
    The `SpriteContents`
    upload-argument
    difference is GPU
    plumbing (the port
    uploads via wgpu
    directly).
  - Verification: no Rust
    change; the tree state
    is the one `cargo
    check -j 1 -p
    horizon-client --lib
    --tests` passed at
    item 155.
- **Item 159 — ValueOutput
  API verified absent
  (1.21.6 only):**
  - `ValueOutput` and
    `TagValueOutput` exist
    only in 1.21.6; 1.21.5
    saves through
    `CompoundTag`
    directly. The Rust
    tree has no
    `ValueOutput` /
    `TagValueOutput`
    equivalent — every
    save path writes NBT
    directly via `Tag` /
    `Compound` (items
    109, 111, 112, 123,
    126, 150, 156). The
    single mention is a
    comment in
    `block_entity_nbt.rs`
    contrasting 1.21.5's
    `optionalFieldOf`
    (empty component map
    omitted) with 1.21.6's
    unconditional store —
    verified against the
    1.21.5 decomp
    (`COMPONENTS_CODEC`,
    `saveWithoutMetadata`)
    and left as is.
  - Verification: no Rust
    change; the tree state
    is the one `cargo
    check -j 1 -p
    horizon-client --lib
    --tests` passed at
    item 155.
- **Item 160 — Desert
  ambient sounds back to
  the 1.21.5 single tag;
  enchanting table and
  pot extents verified:**
  - 1.21.6 split the
    desert ambient tag
    into three
    (`triggers_ambient_desert_*`);
    1.21.5 has the one
    `plays_ambient_desert_block_sounds`
    tag. `animate_tick.rs`
    now loads that single
    table and plays the
    1.21.5 sounds:
    `playAmbientBlockSounds`
    (tag member under open
    sky; idle 1 in 1600,
    wind 1 in 10000 in a
    desert or badlands
    biome, played at the
    player, draws in
    vanilla's order), with
    the simple surrounding
    check (three of the
    four surface columns
    eight blocks away
    carry a tag block —
    1.21.6's form needs
    all four and scans
    vertically). Terracotta
    ticks the sounds too
    (new `Terracotta`
    kind); the `Falling`
    kind lost its sand
    flag (the tag decides).
    Dead bush: 1 in 150
    with the 4-in-5
    red-sand/terracotta
    silence (was 1 in 130
    / 2 in 3). The dry
    grass ambient sound
    is gone
    (`ShortDryGrassBlock`
    / `TallDryGrassBlock`
    have no animate tick
    in 1.21.5). The dried
    ghast branch stays
    inert (its tag is
    absent from the 1.21.5
    data, so the table is
    all false, and the
    block is not in the
    tables — item 144).
    `HAPPY_GHAST_AVOIDS`
    has no surface. The
    rest of `BlockTags`
    is a reorder.
    Enchanting table: the
    save already writes
    `CustomName` only
    when present and the
    parts table strips it
    (`NAME`) — the 1.21.5
    `CompoundTag` form.
    `DecoratedPotRenderer.getExtents`
    (1.21.6-only) has no
    surface (item 138).
    Pre-existing gap,
    version-neutral: the
    port gives sand
    falling dust, but
    vanilla's
    `SandBlock.animateTick`
    never calls super in
    either version.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib
    --tests` 0/0 (1m 22s);
    the classify test now
    pins terracotta ticking
    and dry grass not
    ticking. The
    test-profile build was
    not run (OOM under
    load, item 122).
- **Item 161 — Dimension
  cloud height absent;
  legacy dimension ids
  parsed:**
  - `DimensionType.cloud_height`
    is 1.21.6-only: the
    Rust `DimensionType`
    has no such field and
    the registry reader
    never reads it, so
    there is nothing to
    remove. Cloud height
    comes from the
    `visual/cloud_height`
    default 192.33, which
    matches 1.21.5's
    overworld form
    (`getCloudHeight()`
    192 + 0.33 in
    `LevelRenderer`).
    `parseLegacy` (1.21.5)
    was a real gap: the
    player load read
    `Dimension` as a
    string only, so a
    legacy numeric id
    (-1 nether, 1 end)
    silently loaded as
    the overworld. The
    load now maps -1/0/1
    to the three
    dimensions, takes a
    string as the key,
    and keeps the
    overworld default
    otherwise — pinned by
    a test. No datafix
    converts the field,
    which is why vanilla
    keeps `parseLegacy`
    in `PlayerList`.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib
    --tests` 0/0 (49s).
    The test-profile
    build was not run
    (OOM under load,
    item 122).
- **Item 162 — Entity:
  exact-position flag
  removed; fire, leash
  and interact already
  1.21.5:**
  - `requiresPrecisePosition`
    is 1.21.6-only, and
    it was live: grounded
    primed TNT set the
    Rust `precise` flag
    and forced a full
    position sync every
    tick. The field, the
    tracker term and all
    six construction
    sites are gone; the
    test now pins a
    grounded TNT moving
    by delta. (Item 104's
    "never true in
    production" was
    wrong — the TNT
    setter was missed
    there.) Fire is
    already the 1.21.5
    form: default -1,
    the `wasOnFire &&
    ticks <= 0` reset
    to -1, and the
    fire-immune branch
    unreachable (contact
    never ignites an
    immune mob). The
    leash interact is
    already 1.21.5 — no
    sneak-tie, no
    shears branch, no
    tie/untie sounds —
    and `canAttachLead`
    is the no-distance
    form; equipment
    shearing has no
    surface (snow golem
    and bogged only).
    The leash offset
    default is
    `(0, eyeHeight,
    width * 0.4)`; the
    spawn helper is the
    float-Y form only;
    `checkSlowFallDistance`
    is the name the
    code already cites.
    No surface at all:
    movement recording,
    `isInClouds`,
    `teleportSpectators`
    (the spectator
    camera just ends
    when its target
    goes), the quad
    leash, `isFlyingVehicle`,
    `getCommandSenderWorld`,
    the waypoint hooks.
    Save/load is the
    usual
    `ValueInput`/`ValueOutput`
    → `CompoundTag`
    refactor with the
    same fields (motion
    clamp 10.0 present);
    the `Equippable`
    shear fields live
    only in the pinned
    internal layout,
    with the 1.21.5
    adapter inserting
    and stripping them
    (item 66's ground).
    Pre-existing gap,
    version-neutral:
    a fire-immune mob
    that somehow holds
    fire ticks is
    cleared at once
    instead of decaying
    by 4 a tick.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib
    --tests` 0/0 (1m
    06s). The
    test-profile build
    was not run (OOM
    under load, item
    122).
- **Item 163 —
  Inventory screen
  verified:**
  - The title is
    `rgba(64, 64, 64,
    255)` — the opaque
    spelling of 1.21.5's
    `4210752`, same
    pixels via
    `adjustColor`
    (items 84, 106).
    The effects panel
    draws after the
    slots and recipe
    book, with its
    tooltip last — the
    1.21.5 single
    `render` after
    `super.render`, not
    1.21.6's split
    `renderEffects` /
    `renderTooltip`.
    The player figure
    is (26, 8)–(75,
    78) at scale 30,
    0.0625 — identical
    in both versions.
    The `EntityRenderState`
    submit vs the
    direct pose-stack
    render is GPU
    plumbing with no
    Rust surface (the
    port draws the
    preview itself).
  - Verification: read
    only; no code
    changed.
- **Item 164 — Death,
  loading overlay,
  buffet, level
  loading verified:**
  - Death screen:
    title (double size),
    cause at y 85 and
    score at y 100 are
    all `u32::MAX` —
    the opaque spelling
    of 1.21.5's
    `16777215`. Exit to
    title goes through
    `close_world` →
    `leave_server(None)`
    (no quit message,
    the 1.21.5
    `disconnect()`
    form) or the
    `menu.savingLevel`
    screen. Clickable
    cause-of-death
    links are not
    implemented in the
    port at all, so
    the 1.21.5/1.21.6
    click-path
    difference has no
    surface (one stale
    `disconnectWithSavingScreen`
    comment fixed).
    Loading overlay:
    the port's is
    fully opaque and
    never renders the
    screen behind, so
    `render` vs
    `renderWithTooltip`
    has no surface;
    logo halves and
    the 0.8325 bar
    match both
    versions. Buffet
    biome entries draw
    the name at x+5 in
    `u32::MAX`;
    level-loading
    progress text is
    `u32::MAX` too.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib`
    0/0 (47s).
- **Item 165 —
  FriendlyByteBuf
  verified:**
  - The only
    `readJsonWithCodec`
    caller in the
    1.21.5 decomp is
    the status-response
    packet, and the
    port parses that
    JSON with strict
    `serde_json::from_str`
    — the 1.21.5
    `GsonHelper` form,
    not 1.21.6's
    lenient parser.
    Wire text
    components are
    NBT in the port
    (`Text { text: Tag
    => Nbt }`), as in
    1.21.5. The
    `parse_lenient`
    uses in the tree
    are all datafix
    and options-file
    paths, where
    vanilla is lenient
    too. The
    `readEither` /
    `writeEither`
    helpers (1.21.6
    only) have no
    Rust equivalent
    and no caller.
    The "Failed to
    decode json"
    capitalisation is
    a `DecoderException`
    message, not
    observable on the
    wire.
  - Verification: read
    only; no code
    changed.
- **Item 166 —
  StreamCodec
  composite, duplicate
  of item 147:**
  - This item's diff
    is byte-identical
    to item 147's (the
    11-arity
    `composite`,
    1.21.6-only), and
    item 147 already
    verified it: the
    Rust protocol
    crate has no
    composite
    combinator, and no
    1.21.6 changed
    class calls the
    11-arity form.
    Nothing further to
    do.
  - Verification: `diff`
    of the two item
    files, identical;
    see item 147.
- **Item 167 —
  ItemRenderer
  verified:**
  - `SPECIAL_FOIL_*` /
    `COMPASS_FOIL_*`
    is a rename with
    the same values
    (0.5, 0.75,
    0.0078125); the
    port has no such
    constants (glint
    math uses
    `Glint.scale`).
    The compass-foil
    translucent glint
    (1.21.6) vs the
    always-opaque one
    (1.21.5) is
    unrepresentable
    in the port's
    single-pass
    composite — the
    shader applies
    the same
    `glint_over` math
    for overlays 7
    and 33, and the
    blend follows the
    quad's own
    translucency —
    and vanilla
    compass/clock
    sprites have no
    semi-transparent
    pixels, so it is
    unobservable with
    vanilla assets.
    `getFoilBuffer`
    is a pure refactor
    (identical
    condition). The
    GUI slot centre
    is `translate(x +
    8, y + 8)`; the
    decoration blit
    offset has no
    surface (HUD draw
    order puts
    decorations on
    top). No
    model-identity
    concept anywhere
    (items 121, 138).
  - Verification: read
    only; no code
    changed.
- **Item 168 — Beacon
  screen verified:**
  - Labels are
    `0xFFE0_E0E0` at
    cx 62/169, y 10 —
    the opaque
    spelling of
    1.21.5's
    `14737632`. The
    five payment
    items draw after
    the background
    at x 20/41/63/86/
    108, y 109, in
    vanilla's order
    (netherite first);
    the 1.21.5 z=100
    translate has no
    surface (HUD draw
    order). Effect
    icons are the
    `mob_effect/<name>`
    sprite at +2,+2
    scaled 18x18,
    matching
    `renderIcon`;
    the
    ResourceLocation
    vs
    TextureAtlasSprite
    difference is
    plumbing.
  - Verification: read
    only; no code
    changed.
- **Item 169 — /loot
  and /forceload
  verified:**
  - Both `loot_table`
    arguments are
    `ResourceOrId("minecraft:loot_table")`,
    and the
    suggestion engine
    suggests that
    registry's ids
    for it — the same
    list 1.21.5's
    explicit
    `SUGGEST_LOOT_TABLE`
    produces (and
    1.21.6's built-in
    argument
    suggestions too).
    Both commands
    require level 2
    (`GAMEMASTERS`);
    the
    `hasPermission(2)`
    → lambda change
    is the pure
    refactor seen in
    items 110/134/
    142/145.
  - Verification: read
    only; no code
    changed.
- **Item 170 — Frog
  variant already
  1.21.5; amphibious
  navigation does
  not walk:**
  - Frog variant
    selection uses
    the per-type
    `VariantRegistry::select`
    form and the save
    keeps the variant
    name (items 112,
    127's pattern);
    save/load is the
    usual `CompoundTag`
    refactor.
    `canNavigateGround`
    does not exist in
    1.21.5 at all —
    the door code uses
    `GoalUtils.hasGroundPathNavigation`
    (`instanceof
    GroundPathNavigation`),
    and
    `AmphibiousPathNavigation`
    extends
    `PathNavigation`
    directly. The
    port's
    `can_navigate_ground`
    wrongly returned
    true for
    amphibious (the
    1.21.6 override);
    the arm is
    removed, pinned
    by a test. The
    callers are door
    code no amphibious
    mob runs, so this
    is unobservable in
    practice — but
    the code now
    states the 1.21.5
    form.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib
    --tests` 0/0 (1m
    09s). The
    test-profile build
    was not run (OOM
    under load, item
    122).
- **Item 171 — Book
  view narration
  removed; container
  screens verified:**
  - BookViewScreen in
    1.21.5 has
    `NO_TITLE` and no
    narration
    override, but the
    port narrated the
    1.21.6 form
    (title + page
    indicator + page
    text). The view
    screen's
    `ScreenInfo` now
    carries `""`
    (same shape as
    item 155); the
    page buttons stay
    narratable
    widgets, and the
    now-dead
    `book_view`
    helper and its
    test assertion are
    gone. RunCommand
    links already do
    the 1.21.5 form
    (book closes,
    command runs
    unsigned; lectern
    closes its menu);
    one stale
    `closeContainerOnServer`
    comment fixed.
    Page text and
    indicator are the
    opaque `BLACK`
    spelling at
    vanilla's
    positions
    (x+36, y 32+9/line;
    indicator at
    left+148-width,
    y 18). Container
    screens: labels
    draw before slots
    in the port (the
    1.21.6 order) but
    labels never
    overlap slots, so
    it is
    unobservable;
    tooltips render
    in-frame (the
    1.21.5 form, not
    1.21.6's
    next-frame defer);
    the snapback
    animation is not
    implemented at
    all (version-neutral
    gap), so the
    record-vs-fields
    difference has no
    surface; blit
    offsets and
    strata are
    plumbing (HUD
    draw order).
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib
    --tests` 0/0 (45s).
    The test-profile
    build was not run
    (OOM under load,
    item 122).
- **Item 172 — Command
  tree levels come
  from the tree
  (ArgumentUtils):**
  - 1.21.5's
    `commands.json`
    carries no
    permission info at
    all — the
    `required_level`
    field only appears
    in 1.21.6 (verified
    against the real
    reports: 1.21.5's
    `op`/`ban`/`tp`
    have no level key,
    1.21.6's have
    3/3/2) — but the
    port's reader only
    understood the
    26.x `permissions`
    object, so every
    node read as level
    0 and the LAN
    server sent the
    whole tree, /op
    and /stop included,
    to level-0 guests.
    A silent vanilla
    root now takes its
    level from the
    tree's own
    registrations
    (`vanilla::root_permission`,
    built per server
    selection so it
    does not depend on
    the process's
    server-kind
    atomics); a report
    that does carry a
    level still wins,
    and the reader now
    also parses the
    1.21.6 integer
    form. Mod roots and
    the pregen tree
    are untouched (the
    pregen tree carries
    its own
    permissions).
    `findUsedArgumentTypes`
    has no surface
    (the port reads
    argument types
    from the registry);
    the rest of the
    diff is refactor
    (switch to
    instanceof,
    `isEmpty` to
    `size() > 0`) over
    identical JSON.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib
    --tests` 0/0 (58s);
    `cargo test -j 1 -p
    horizon-command
    --lib
    root_permission` 1
    passed (6m; 23
    levels pinned
    against the 1.21.6
    report). The
    horizon-client
    test-profile build
    OOM-killed under
    3-stream load
    (full features,
    ~20m in) and the
    `--no-default-features`
    retry was killed
    at ~23m to spare
    the other streams'
    builds, so the
    `lan::commands`
    tests (new
    level-free-report
    test, updated
    filter test) run
    in CI.
- **Item 173 — NeutralMob
  anger saves the time
  left (ZombifiedPiglin,
  AbstractSkeleton):**
  - The diff is the
    ValueInput/ValueOutput
    to CompoundTag
    refactor, but under it
    the port wrote the 26.x
    form: `anger_end_time`
    (absolute, long) and
    `angry_at`, where
    1.21.5's `NeutralMob`
    saves `AngerTime` (int,
    remaining) and
    `AngryAt` (verified in
    the 1.21.5 decomp). A
    vanilla save's anger
    was lost on load, and
    `/summon wolf ~ ~ ~
    {AngerTime:400}` did
    nothing. The save/load
    now reads and writes
    the 1.21.5 keys; the
    model keeps its
    absolute end time and
    the conversion happens
    at the boundary only
    (`adopt_anger_end`
    derives the end from
    the time left;
    `refresh_anger_for_save`
    recomputes the time
    left from the end
    before a save, so
    missed ticks count).
    The AbstractSkeleton
    half needs no change:
    the port reassesses the
    weapon goal on its
    first check after load.
    The wolf's synched
    metadata (absolute end)
    is unchanged. Known
    follow-up: the bee
    still writes
    `anger_end_time` — its
    own model needs the
    clock threaded into
    `bee::to_nbt`/`from_nbt`.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib
    --tests` 0/0 (1m52s).
    The test-profile build
    was not run (OOM under
    load, item 122), so the
    reworked h262 / h13076
    / live-wolf tests run
    in CI.
- **Item 174 — Adventure
  predicate, removed
  items, dye text
  colours:**
  - AdventureModePredicate:
    the port tests
    `BlockPredicate.matches`
    directly (state, then
    `nbt` against the block
    entity's tag) and has
    no `areSameBlocks`
    cache at all, so the
    diff — the cache
    helper's `saveWithId`
    form replacing the
    `TagValueOutput` one,
    over identical tags —
    has no surface. Items:
    the registry is
    data-driven from the
    1.21.5 prepared
    resources, and no Rust
    or Python in the tree
    names `dried_ghast`,
    any `*_harness`,
    `happy_ghast_spawn_egg`
    or `music_disc_tears`;
    jukebox songs resolve
    by id from the same
    registry. The carved
    pumpkin and the seven
    heads carry no
    waypoint-hide attribute
    in the port (only the
    pumpkinblur overlay);
    the waypoint attribute
    hooks belong to items
    13/225/260/264.
    DyeColor: `DYE_TEXT`
    already holds all
    sixteen 1.21.5 raw
    `textColor` values,
    checked against the
    decomp.
  - Verification: read
    only; no code
    changed.
- **Item 175 — Click
  actions lose
  `show_dialog` and
  `custom`:**
  - Both actions arrived
    with dialogs in 1.21.6,
    so the `ClickEvent`
    variants, their parse
    and serialize arms, and
    the chat, book and
    dialog-text handlers
    are gone; an unknown
    action now falls
    through to the drop
    arm, keeping the text.
    The dialog model and UI
    stay: dialog-internal
    clicks still use them,
    and items 219/222/258
    own the registry and
    client-side removal
    (`send_custom_click`
    stays live via the
    dialog click handler).
    `NbtAccounter`: the
    Rust NBT reader has no
    accounter or quota at
    all, so the 1.21.6
    negative-size throw has
    no surface.
  - Verification: `cargo
    check -j 1 -p
    horizon-client-core -p
    horizon-client --lib
    --tests` 0/0 (2m31s);
    `cargo test -j 1 -p
    horizon-client-core
    --lib
    one_twenty_one_six` 1
    passed.
- **Item 176 — Section
  render dispatcher
  (1.21.5 mesh model):**
  - The 1.21.6
    `SectionMesh` rewrite
    has no surface: the
    port meshes its own
    way, and the observable
    parts already match
    1.21.5 — translucency
    resorting is the
    point-of-view plus
    axis-aligned rule over
    a stable by-distance
    sort, an empty section
    is all-visible, and the
    stats string is
    identical. Known
    deviation, recorded at
    the gate: 1.21.5
    exempts sections within
    24 blocks of the camera
    from the neighbour
    check, but the port
    keeps its stricter
    lit-neighbours gate
    everywhere (Streaming
    knows only the chunk
    centre, not the camera,
    and meshing an unlit
    column would draw dark
    borders until the
    relight remesh), so
    near sections at a load
    edge wait here where
    vanilla draws them at
    once.
  - Verification: `cargo
    check -j 1 -p
    horizon-client-render
    --lib` 0/0 (38s);
    comment only.
- **Item 177 — Recipe
  book ghost slots,
  overlay, tabs:**
  - Ghost slots draw the
    exact vanilla fills
    (822018048 behind,
    822083583 over the fake
    item, decorations for
    the result); the
    render-type difference
    is pipeline plumbing
    over the same colour.
    The ghost tooltip draws
    in-frame from
    `draw_slot_tooltip`,
    the 1.21.5 form, not
    the 1.21.6 next-frame
    defer. The overlay's
    sprite size, 25px
    button grid and 0.375
    mini-item scale match,
    and its z=1000 / z=150
    layering is the port's
    draw order. Tab squeeze
    is the same formula
    about (x+8, y+12),
    selected tabs shift -2.
  - Verification: read
    only; no code
    changed.
- **Item 178 — Ender
  chest save/load,
  shulker screen:**
  - The ender chest already
    saves and loads the
    1.21.5 form:
    `decode_items` clears
    all 27 slots, then
    fills from the list
    with the `Slot` byte
    masked to 0..255 and a
    range check, a
    malformed stack
    becoming empty;
    `encode_items` writes
    `Slot`-tagged compounds
    for non-empty slots
    only. The
    ValueInput/ValueOutput
    names have no Rust
    surface (item 159). The
    shulker screen diff is
    pipeline naming over
    identical blit
    arguments, and shulker
    boxes open through the
    generic container path.
  - Verification: read
    only; no code
    changed.
- **Item 179 — Cow
  save/load and
  variant selection:**
  - The save/load is the
    ValueInput/ValueOutput
    to CompoundTag refactor
    over the identical
    `variant` name, which
    the port already writes
    and reads for cows.
    Spawn selection is the
    1.21.5 per-type form:
    `VariantRegistry::select`
    takes the
    highest-priority
    entries whose condition
    holds and picks one at
    random, the same pick
    1.21.6's generic picker
    makes.
  - Verification: read
    only; no code
    changed.
- **Item 180 — Parrot
  imitation map:**
  - The imitation table is
    back to vanilla
    1.21.5's 37 entries:
    the ender dragon is in,
    the happy ghast and the
    26.x additions (camel
    husk, parched, zombie
    horse, zombie nautilus)
    are out. The dragon has
    no `Species` (a
    separate boss entity),
    so its key is `None`
    and it feeds only the
    random ambient draw, as
    in vanilla; a parrot
    never imitates a nearby
    dragon, a known gap.
    The `Variant` save/load
    was already the 1.21.5
    int form.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib
    --tests`, 0 errors, 0
    warnings.
- **Item 181 — Ender
  dragon attributes
  and save/load:**
  - The dragon saves and
    loads `DragonPhase` and
    `DragonDeathTime` as
    ints already, the phase
    only when present (a
    save without it keeps
    the hovering default)
    and the death time
    defaulting to 0. The
    1.21.6
    `camera_distance`
    attribute has no
    surface: the dragon
    carries no attributes
    at all, and
    `Camera.setup` is
    already the 1.21.5 flat
    4 times the scale,
    ignoring the
    1.21.6-only input the
    camera structs still
    carry.
  - Verification: read
    only; no code
    changed.
- **Item 182 — Nether
  portal slab, cauldron
  content, dried ghast:**
  - The nether portal
    inside test is the
    1.21.5 slab now, a 4/16
    slice across the cell,
    at all three sites:
    swept entities
    (`touches_nether_portal`),
    guest players
    (`nether_portal_inside`)
    and the local player
    (`check_inside_blocks`).
    1.21.6 dropped the
    override and fell back
    to the whole block.
    Nether fossils no
    longer place a dried
    ghast
    (`placeDriedGhast` is
    1.21.6-only); the
    remaining dried-ghast
    references are inert.
    The cauldron content
    test was already the
    1.21.5 form for the
    player and item
    entities; mobs have no
    cauldron path, a
    version-neutral gap.
    Terracotta's desert
    sounds were already
    ported (item 160).
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib
    --tests -p
    horizon-world`, 0
    errors, 0 warnings;
    standalone rustc run
    of the 5 slab
    assertions passed;
    lib test build
    OOM-killed under
    load, test runs in
    CI.
- **Item 183 — Chunk
  section empty,
  container size:**
  - `isSectionEmpty` is a
    trivial accessor 1.21.6
    inlined away; the
    port's section checks
    already use the
    `hasOnlyAir` form. The
    `ProblemReporter` chunk
    path is 1.21.6-only
    diagnostics with no
    Rust surface. The
    container size formula
    differs only as an
    estimate: 1.21.5
    overcounts by a VarInt
    the wire form never
    carries (`write` uses a
    fixed-size long array
    in both versions), and
    the port measures
    actual encoded bytes,
    with no size function
    at all. Its
    `Paletted::write`
    matches the shared wire
    form.
  - Verification: read
    only; no code
    changed.
- **Item 184 — Realms
  disconnect, profile
  key pair file:**
  - The port has one
    disconnect screen with
    the same title, wrapped
    reason and Back button
    as both vanilla
    screens. Its Back
    always returns to the
    server list where
    vanilla returns to the
    parent (the Realms
    screen after a Realms
    join) in both versions,
    a version-neutral gap.
    The profile key pair
    file is never read or
    written: the chat keys
    come from the session
    service and stay in
    memory, so the JSON
    parser change has no
    surface.
  - Verification: read
    only; no code
    changed.
- **Item 185 — Skin
  texture download
  and insecure skin:**
  - Both diffs are pure
    refactors with
    identical behaviour:
    the texture
    registration only
    inlines a local, and
    the new two-argument
    `getInsecureSkin`
    overload just names
    what the one-argument
    form already did (a
    loaded skin or the
    default). The port
    resolves skins through
    its own async cache
    with the default-skin
    fallback per call site,
    and has no two-argument
    form to match.
  - Verification: read
    only; no code
    changed.
- **Item 186 — Creaking
  home position
  save/load:**
  - The creaking already
    saves and loads
    `home_pos` the 1.21.5
    way: the writer stores
    it as an int array only
    when the heart binding
    exists and removes the
    key otherwise, and the
    reader calls
    `set_transient` only
    when a three-entry
    array is present. The
    ValueInput/ValueOutput
    names have no Rust
    surface (item 159).
  - Verification: read
    only; no code
    changed.
- **Item 187 — Endermite
  lifetime save/load:**
  - The endermite already
    saves `Lifetime` as an
    int and reads it back
    defaulting to 0, and
    its life tick is the
    vanilla form: the
    counter climbs while
    the mite is not
    persistence-required
    and it is discarded at
    2400. The
    ValueInput/ValueOutput
    names have no Rust
    surface (item 159).
  - Verification: read
    only; no code
    changed.
- **Item 188 — Goat
  horns and screaming
  save/load:**
  - The goat already saves
    `IsScreamingGoat` and
    both horn flags, and
    reads them with
    vanilla's defaults:
    screaming false when
    absent, and a horn kept
    unless its key is
    present and zero. The
    ValueInput/ValueOutput
    names have no Rust
    surface (item 159).
  - Verification: read
    only; no code
    changed.
- **Item 189 — Panda
  genes save/load:**
  - The panda already
    writes `MainGene` and
    `HiddenGene` as gene
    name strings and reads
    them back through the
    gene table, falling
    back to normal (index
    0) for an absent or
    unknown name, as
    vanilla's codec default
    does. The
    ValueInput/ValueOutput
    names have no Rust
    surface (item 159).
  - Verification: read
    only; no code
    changed.
- **Item 190 — Sculk
  spreader cursors
  save/load:**
  - The catalyst already
    saves its charge
    cursors as a list of
    compounds with pos,
    charge, decay and
    update delays and
    optional facings, and
    loads at most 32 of
    them, skipping
    malformed entries and
    clamping the fields to
    the codec's ranges; a
    missing key leaves an
    empty spreader. The
    ValueInput/ValueOutput
    names have no Rust
    surface (item 159).
  - Verification: read
    only; no code
    changed.
- **Item 191 — Create
  world screen
  renames:**
  - All three changes are
    renames with identical
    behaviour: the blit
    pipeline argument, and
    the version stability
    and name accessors. The
    port's own draws match
    the shared blit args
    (tab header 16 tile,
    footer separator 32x2
    at h-footer-2). Known
    gap, version-neutral:
    the port shows the
    Experiments button
    always, while vanilla
    1.21.5, a stable
    release, hides it
    behind the stability
    gate.
  - Verification: read
    only; no code
    changed.
- **Item 192 — SNBT
  underscore error
  key typo:**
  - The SNBT parser used
    1.21.6's corrected
    error key spelling;
    1.21.5 spells it
    `undescore_not_allowed`,
    and that typo is what
    the 1.21.5 lang file
    carries, so the key is
    back to the typo'd
    form. The NbtUtils half
    is a rename plus
    ValueOutput overloads
    with no Rust surface
    (item 159); DataVersion
    stamping is already
    4325.
  - Verification: cargo check -j 1 -p
    horizon-command
    --all-targets 0/0;
    cargo test -p
    horizon-command --lib
    snbt 7/7.
- **Item 193 — Tadpole
  age save/load;
  no age lock:**
  - The tadpole saves and
    loads only `Age` now,
    and the same fix
    reaches every ageable
    mob: 1.21.5 has no age
    lock at all (no field
    or setter on AgeableMob
    in either version,
    confirmed by javap), so
    `AgeLocked` is no
    longer written, read,
    or carried in axolotl
    and tadpole bucket
    tags, and a stale key
    from a raw tag is
    dropped. Item 10 missed
    this. The
    golden-dandelion lock
    interaction stays in
    memory only; removing
    the mechanic belongs to
    no item, since it
    exists in neither
    version.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib
    --tests 0/0; lib test
    build OOM-killed under
    load, tests run in CI.
- **Item 194 — Wither
  invulnerability
  save/load:**
  - The wither already
    saves `Invul` as an int
    and reads it back
    defaulting to 0. The
    ValueInput/ValueOutput
    names have no Rust
    surface (item 159).
  - Verification: read
    only; no code
    changed.
- **Item 195 — Villager
  offers and
  inventory save/load:**
  - The villager already
    writes `Offers` in the
    codec form, `Xp`,
    `LastRestock`,
    `RestocksToday`,
    `VillagerData` and its
    eight-slot `Inventory`,
    and reads them back
    with the unset
    detection. The 1.21.5
    form threads the
    serialization context
    explicitly where
    1.21.6's ValueOutput
    carried it; the port
    writes NBT directly and
    has no such context to
    thread. The
    ValueInput/ValueOutput
    names have no Rust
    surface (item 159).
  - Verification: read
    only; no code
    changed.
- **Item 196 — Hurting
  projectile
  acceleration save/load:**
  - Fireballs and wither
    skulls already save
    `acceleration_power` as
    a double and read it
    back defaulting to 0.1.
    The
    ValueInput/ValueOutput
    names have no Rust
    surface (item 159).
  - Verification: read
    only; no code
    changed.
- **Item 197 — Collision
  context: no
  withPosition:**
  - `withPosition` arrived
    in 1.21.6, so it is
    gone: the
    `with_position`
    constructor is removed
    and `placement` builds
    the context directly,
    taking any entity as
    1.21.5's widened
    parameter allows. The
    four `with_*` modes are
    removed from the parity
    fixture, the Java
    oracle (which could not
    compile against the
    1.21.5 jar while it
    called the method) and
    the five collision
    drivers. No production
    code called it.
  - Verification: `cargo
    check -j 1 -p
    horizon-world
    --all-targets 0/0;
    drivers py_compile; no
    unit tests cover the
    module; oracle edit
    uncompiled here (no
    full 1.21.5 jar).
- **Item 198 — ValueInput
  removed (1.21.6
  only):**
  - `ValueInput` is a
    1.21.6-only class: the
    Rust port has no
    equivalent (item 159)
    and needs none. The
    food oracle, the one
    in-subset harness on
    the save API, is
    converted to the 1.21.5
    form: `FoodData` takes
    a `CompoundTag`
    directly, so the
    `TagValue` wrappers are
    gone from the oracle
    and its driver scope
    string. Eleven more
    oracles still use the
    1.21.6 save API; all
    out of subset, recorded
    as publish debt.
  - Verification: javac
    shows no syntax errors
    (missing-symbol errors
    only, no 1.21.5 jar
    here); driver
    py_compile; full oracle
    compile runs in CI.
- **Item 199 — Evoker
  empty save/load
  overrides:**
  - 1.21.5's evoker
    overrides are empty:
    they only call super,
    and 1.21.6 deleted them
    as dead code. The port
    matches: the evoker
    saves nothing of its
    own, and its
    `SpellTicks` timer
    comes from the shared
    spellcaster path. The
    empty synched data
    override has no Rust
    surface.
  - Verification: read
    only; no code
    changed.
- **Item 200 — Ground
  nav min-Y bound
  (strict):**
  - The ground path scan is
    back to 1.21.5's strict
    min-Y bound: the
    down-scan and its check
    use `>` now, so a floor
    at exactly the world's
    bottom is not a path
    target (1.21.6 widened
    both to `>=`). The
    other two changes were
    already the 1.21.5
    form:
    `can_navigate_ground`
    is the `instanceof`
    check (item 170), and
    the nav
    `can_open_doors` flag
    is the
    `setCanOpenDoors`
    mechanism.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib
    --tests 0/0; no tests
    cover the boundary.
- **Item 201 — CustomData
  loadInto:**
  - Both `loadInto` paths
    are the same algorithm
    in both versions: save,
    merge the custom tag,
    load back (the entity
    keeps its UUID; the
    block entity loads only
    when the merge changed
    anything, rolling back
    on failure). Only the
    plumbing differs
    (CompoundTag vs
    ValueOutput), and the
    ProblemReporter wrapper
    is diagnostics with no
    Rust surface. The
    port's block-entity
    `place` already
    implements the same
    steps, and its merge is
    the recursive
    `CompoundTag.merge`
    form.
  - Verification: read
    only; no code
    changed.
- **Item 202 — Pack
  listing has no
  .DS_Store filter:**
  - 1.21.6's pack listing
    excludes `.DS_Store`,
    but only when running
    in an IDE; 1.21.5 lists
    every regular file. The
    port's pack walker
    lists every
    non-directory entry
    with no name filter,
    and a native binary has
    no running-in-an-IDE
    concept for the 1.21.6
    branch to hang on.
  - Verification: read
    only; no code
    changed.
- **Item 203 — Rail
  rotate/mirror
  helpers:**
  - 1.21.6 factored the
    rail shape tables into
    helpers; 1.21.5 inlines
    them, with one real
    difference: its
    180-degree arm has no
    straight-rail cases and
    falls through to the
    90-degree arm, so a
    straight rail rotated
    180 degrees swaps axis
    (confirmed by javap on
    the 1.21.5 jar). The
    port needs no change:
    its rotate table is
    exported by calling
    vanilla's own
    `BlockState.rotate` on
    the 1.21.5 server jar,
    so the fallthrough is
    in the data. The mirror
    tables are identical in
    both versions.
  - Verification: read
    only; no code
    changed.
- **Item 204 — Chat
  narration:**
  - 1.21.5 says player chat
    with `sayChat`, system
    messages once with
    `say` after the branch,
    and never narrates a
    chat validation error.
    The port already used
    those calls, but it
    gated narration on the
    line showing:
    `signed_chat_line`
    returned false when the
    Chat option hid the
    source, which also
    dropped the chat log,
    the processed mark and
    the delay timer.
    Vanilla narrates, logs
    and marks accepted
    messages whether or not
    they show, so the early
    return is gone —
    visibility filters only
    the display, in
    `push_message` — and
    system chat says once
    after the branch. Stale
    `sayChatQueued` /
    `saySystemChatQueued`
    comments corrected.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib` 0/0
    (28m58s, contended).
- **Item 205 —
  CommandSourceStack:**
  - The `PermissionSource`
    interface is
    1.21.6-only plumbing:
    `hasPermission` keeps
    the same
    `permissionLevel >=
    level` body in both
    versions, and the port
    enforces permission at
    the tree level, with no
    command-source
    `hasPermission` at all.
    The suggestion change
    is the same shape:
    1.21.6's `getLookup`
    falls back to the
    reloadable registries,
    1.21.5 reads
    `registryAccess` only,
    and the port suggests
    from a single merged
    registry table with no
    reloadable layer and no
    fallback.
  - Verification: read
    only; no code
    changed.
- **Item 206 —
  Reloadable registries:**
  - The logging change
    looks real but is not:
    1.21.5's problem
    collector stores plain
    description strings
    (javap:
    `Multimap<String,
    String>`,
    `report(String)`), so
    logging the entry
    prints the same text
    1.21.6 gets from
    `description()` — the
    port's
    recursive-reference
    diagnostic matches
    both. The `lookup`
    return type is the
    `HolderGetter` /
    `HolderLookup` rename,
    and 1.21.5's `getKeys`
    is the element-id
    listing the port
    already has as
    `Registries::ids`.
  - Verification: read
    only; no code
    changed.
- **Item 207 — Brain
  entity tracker:**
  - 1.21.6's
    `targetEyeHeight` third
    argument lets
    `currentBlockPosition`
    return the eye block;
    1.21.5 always returns
    the feet block. Both of
    the port's `Tracker`
    enums are already the
    1.21.5 form: the
    two-argument entity
    case (`trackEyeHeight`
    only) and a block
    position taken from the
    feet, with a comment
    recording that the
    eye-height block is
    1.21.6-only.
  - Verification: read
    only; no code
    changed.
- **Item 208 — Player
  head renderer:**
  - 1.21.6 split player
    heads into their own
    `PlayerHeadSpecialRenderer`
    with a `player_head`
    codec id; 1.21.5
    renders them through
    the generic head type.
    The port is already the
    1.21.5 form: one
    `minecraft:head`
    special type with
    `kind` one of the seven
    types including
    `player` (a
    `player_head` type hits
    the unknown-type
    error), and the
    renderer resolves the
    profile skin inline for
    `kind == "player"`,
    with a comment
    recording that 1.21.5
    has no separate
    renderer.
  - Verification: read
    only; no code
    changed.
- **Item 209 — Sound
  source UI:**
  - 1.21.6 added the `UI`
    sound source; 1.21.5
    has ten. The port's
    single `SOUND_SOURCES`
    table holds exactly
    those ten in vanilla
    order, with no `ui`,
    and every use — the
    options sliders, the
    options file, the
    per-source volume
    lookup — reads from it.
  - Verification: read
    only; no code
    changed.
- **Item 210 — ChunkMap
  visibility:**
  - `getUpdatingChunkIfPresent`
    is public in 1.21.6 and
    protected in 1.21.5,
    with an identical body.
    A Java visibility
    change has no Rust
    surface, and the port
    has no
    updating-chunk-map
    accessor by that name
    at all.
  - Verification: read
    only; no code
    changed.
- **Item 211 —
  ColorLerper:**
  - 1.21.6's new
    `ColorLerper` factored
    the jeb_ sheep colour
    cycle out of
    `SheepRenderState` —
    and dropped the
    entity-id offset:
    1.21.5 adds `this.id`
    to the cycle index, so
    two jeb_ sheep do not
    cycle in sync. The port
    had the 1.21.6 form;
    `jeb_wool_rgb` now
    takes the id and the
    call site passes
    `m.id`. The wool colour
    table is identical in
    both versions (white
    -1644826, the rest at
    0.75). The toast half
    has no port surface:
    the port has no Now
    Playing toast.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib
    --tests` 0/0 (3m08s);
    test-profile build
    OOM-killed under load,
    10 assertions verified
    by a standalone rustc
    run; test runs in CI.
- **Item 212 — Recipe
  book page:**
  - 1.21.6 added a
    `nextStratum` before
    the overlay and defers
    the hover tooltip to
    the next frame; 1.21.5
    has no stratum and
    renders the tooltip
    in-frame. The port's
    tooltip was already
    in-frame
    (`draw_tooltip_lines`
    in the same frame), but
    it still emitted a
    `Call::Stratum` — a
    no-op in the game
    renderer, a real entry
    in the probe fixture's
    draws, which would
    mismatch vanilla 1.21.5
    (it never calls it).
    The variant, its
    emission, the match arm
    and the fixture arm are
    all removed; the
    oracle's override
    simply never fires
    against the 1.21.5 jar.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib
    --tests` 0/0 (4m19s);
    `cargo check -j 1 -p
    horizon-client-lib
    --example client_probe`
    0/0 (2m16s).
- **Item 213 — Shearable
  animal armor:**
  - 1.21.6 made wolf and
    horse armor shearable
    (`canBeSheared` plus a
    shearing sound); 1.21.5
    has neither field. The
    port has no
    armor-shearing
    interaction at all —
    hand shearing covers
    sheep, mooshroom, snow
    golem and bogged,
    dispenser shearing the
    same plus sulfur cube
    and copper golem — and
    the 1.21.5 adapter
    forces `canBeSheared`
    false on read and
    strips both fields on
    write, so no item can
    carry the flag.
  - Verification: read
    only; no code
    changed.
- **Item 214 —
  world.waypoints:**
  - The whole package —
    `Waypoint`,
    `TrackedWaypoint`,
    `WaypointTransmitter` —
    is 1.21.6-only. Its
    port surface is already
    gone: the manager,
    packets, command and
    locator bar went in
    items 108/131/145, and
    no hide modifier, icon,
    transmitter logic or
    332-block rule survives
    anywhere. The only
    remnant was a stale
    comment citing the
    deleted
    `lan_waypoints.rs`, now
    removed. The waypoint
    range attributes and
    their update events
    stay: items 225/260/264
    own them. (Bot
    navigation and
    pathfinding waypoints
    are unrelated.)
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib`
    0/0 (3m29s).
- **Item 215 — Recipe
  book component:**
  - The search box text
    colour is 1.21.5's
    alpha-0 16777215
    spelling against
    1.21.6's opaque -1 —
    the same pixels via
    `adjustColor`, and the
    port keeps the opaque
    `u32::MAX` as elsewhere
    (its renderer drops
    fully transparent
    text). The book draws
    after the container
    background, slots and
    sort buttons, which is
    what 1.21.5's z+100
    pose does — the HUD
    batch has no depth,
    order decides. The blit
    pipeline swap has no
    Rust surface.
  - Verification: read
    only; no code
    changed.
- **Item 216 — Client
  entry, quick play:**
  - 1.21.6's
    `QuickPlayVariant`
    (with its only-one
    check) and the optional
    singleplayer argument
    are gone in 1.21.5:
    three plain strings, a
    required value. The
    port already parses
    that way — separate
    `quick_play` / `server`
    fields, a missing value
    is an error, no
    exclusivity check. But
    its startup precedence
    was backwards: server
    first. Vanilla 1.21.5's
    `QuickPlay.connect`
    tries singleplayer,
    then multiplayer, then
    realms (confirmed by
    javap), so both startup
    sites now take the
    world first. Known gap:
    the port has no
    `--quickPlayRealms` at
    all. The Tracy `name()`
    / `getName()` rename
    has no surface — the
    port has no Tracy
    client.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib
    --tests` 0/0 (2m48s).
- **Item 217 — Beacon
  off-screen render:**
  - `shouldRenderOffScreen`
    takes the block entity
    in 1.21.5 and no
    argument in 1.21.6;
    both return true. A
    signature change only —
    the port's never-culled
    set already includes
    the beacon, with the
    end gateway, structure
    block and test instance
    block.
  - Verification: read
    only; no code
    changed.
- **Item 218 — Music
  manager, sound
  engine:**
  - The 1.21.6 music rework
    is absent throughout:
    no `MusicFrequency`
    option, no Now Playing
    toast, the delay logic
    is the plain min/max
    form, the manager
    freezes on pause, pause
    silences every channel,
    zero-volume sounds are
    skipped unless they can
    start silent (no MUSIC
    exception), and the
    tick stops zero-volume
    sources at once — each
    already cited as the
    1.21.5 form in the
    code. Two fixes: the
    countdown checked after
    decrementing, so menu
    music started on tick
    100 where vanilla's
    `nextSongDelay-- <= 0`
    starts it on tick 101;
    and the probe fixture's
    pause op kept the
    1.21.6
    `pauseAllExcept(MUSIC,
    UI)` set, now empty.
    Publish debt: the
    sound-engine oracle
    still calls it (and
    prints `play`'s result,
    which is void in
    1.21.5) — out of
    subset.
  - Verification: `cargo
    check -j 1 -p
    horizon-client --lib
    --tests` 0/0 (1m29s);
    client_probe 0/0 (46s);
    test-profile build
    OOM-killed under load,
    test runs in CI.

- **Item 219 — Minecraft
  (client class):
  - Situational music: the
    port chose music from
    the environment
    attributes only, which
    1.21.5 data never
    carries, so no in-world
    music played at all.
    The 1.21.5 hardcoded
    chain now runs when the
    attributes are empty,
    in vanilla's order: the
    End plays Musics.END,
    underwater music keeps
    playing once started
    and starts underwater
    in a #minecraft:plays_u
    nderwater_music biome,
    creative music plays in
    creative mode outside
    the Nether, else
    Musics.GAME; the
    attribute selection
    still wins when
    present. Known gap: the
    legacy biome music
    field is not read yet,
    so biome-specific
    tracks fall back to
    GAME. The situational-
    music fixture follows
    (it has no tag or
    manager state, so the
    underwater branch never
    fires there); the
    harness is pinned to
    26.3 and out of subset
    — it needs a 1.21.5
    oracle before it can
    run in this tree.
  - Social interactions
    key: in singleplayer
    (unpublished) P now
    shows the not-available
    overlay message and
    narrates it instead of
    opening the screen; the
    pause button still
    opens directly, as
    vanilla's PauseScreen
    does.
  - Verified, no code: no
    quick-actions key or
    dialog lookup (the
    minecraft:quick_actions
    registry entries belong
    to items 222/258); no
    waypoint styles; mob-
    effect icons are atlas
    sprites by effect id in
    both (1.21.5's manager
    is a TextureAtlasHolder
    over textures/atlas/mob
    _effects.png); the
    disconnect screens are
    identical behaviour
    (1.21.5's disconnect()
    is the progress screen;
    the saving screen is
    inlined at the same
    call site); say_now is
    already the 1.21.5
    name; glint strength is
    applied at draw time;
    no panorama screenshot,
    no gametest ticker, no
    LogoRenderer (the title
    draws
    gui/title/minecraft
    directly); the music
    tick gating and pause
    silencing are already
    the 1.21.5 form (item
    218).
  - Verification: `cargo
    check -j 1 -p horizon-
    client --all-targets`
    0/0 (3m02s);
    client_probe 0/0 (49s);
    test-profile build
    killed past 15 min
    under load, the 8 test
    assertions verified by
    standalone rustc run,
    test runs in CI.

- **Item 220 — System
  report, Util:
  - Both hunks are accessor
    renames with identical
    values:
    getName()/getId() for
    name()/id() in the
    system report's version
    details, and getDataVer
    sion().getVersion() for
    dataVersion().version()
    in the datafixer schema
    lookup (plus a line
    rewrap). The port's
    crash report writes
    Minecraft Version from
    the pin
    (VANILLA_VERSION,
    1.21.5); it has no
    Minecraft Version ID
    line and no Java lines
    — its report is its own
    format, the same in
    both versions' terms.
    The datafixer lookup's
    value is DATA_VERSION,
    4325.
  - Verification: read
    only; no code changed.

- **Item 221 —
  CommonComponents:
  - 1.21.6 only factored
    the label choice into
    CommonComponents (the
    GUI_RETURN_TO_MENU
    constant and
    disconnectButtonLabel);
    1.21.5 spells the same
    ternary inline in
    PauseScreen
    (isLocalServer() ?
    RETURN_TO_MENU :
    GUI_DISCONNECT), and
    the port already
    implements exactly that
    at both quit-button
    sites — the pause menu
    (remote ?
    menu.disconnect :
    menu.returnToMenu) and
    the dialog-warning
    screen
    (external_server() the
    same way). The
    disconnect screen's
    button is gui.toMenu,
    untouched by this diff.
  - Verification: read
    only; no code changed.

- **Item 222 — Registry
  data loader:
  - The minecraft:dialog
    registry is gone from
    the loader: removed
    from SYNCHRONIZED (the
    configuration-phase
    registries), and with
    it the only pack
    registry —
    PACK_REGISTRIES,
    pack_entries and the
    sync_dialog_ids
    plumbing are deleted
    (the /dialog command
    went in item 145, so
    nothing read the ids).
    The merge_pack_tags
    dialog special-case is
    gone too; pack tags now
    merge only for
    registries the packs
    have tag files for. The
    dialog UI system itself
    stays for item 258 (its
    registry lookups now
    find nothing, which
    they already handle).
  - Tests: the dialog pack-
    entries test is removed
    with pack_entries; the
    known-pack and cached-
    NBT tests are reworked
    onto
    minecraft:chat_type
    (they pin generic
    machinery); the lan
    reload test drops its
    dialog overlay files
    and assertions (its
    subject is damage
    laziness).
  - Known gap: 1.21.5
    parses registry data
    with Gson's lenient
    parser (JsonParser.pars
    eReader), 1.21.6 with
    the strict one; the
    port parses with strict
    serde_json throughout,
    so a datapack file with
    comments or trailing
    commas loads in vanilla
    1.21.5 but is rejected
    here. No code change —
    a lenient parser is out
    of proportion for this
    item.
  - Verification: `cargo
    check -j 1 -p horizon-
    client --all-targets`
    0/0 (7m56s); reworked
    tests run in CI.

- **Item 223 — Server
  recipe book, stats:
  - The save form is
    already the 1.21.5
    direct-NBT one:
    Settings.write is toNbt
    (all eight flags,
    recipes, toBeDisplayed)
    and read_valid is
    fromNbt. One fix: a
    missing list now reads
    as empty (1.21.5's
    orElse(List.of()))
    instead of dropping the
    whole compound — the
    1.21.6 codec's
    required-field
    strictness. The
    settings-packet copy
    and copyOverData's
    pack/apply round trip
    are unobservable (same
    bytes, same result);
    the port has no copy-
    over path.
    HAPPY_GHAST_ONE_CM has
    no surface: no happy
    ghast anywhere in the
    port.
  - Verification: `cargo
    check -j 1 -p horizon-
    client --lib --tests`
    0/0 (2m05s); new test
    runs in CI.

- **Item 224 — Entity/item
  tag keys:
  - Five tag keys are
    1.21.6-only and none is
    referenced anywhere in
    the port:
    can_equip_harness and f
    ollowable_friendly_mobs
    (entity tags),
    harnesses,
    happy_ghast_food and
    happy_ghast_tempt_items
    (item tags) — the
    harness and happy ghast
    have no Rust surface at
    all (items 144/174).
    The rest of the
    ItemTags diff is
    declaration reordering
    of tags both versions
    have, which has no
    runtime effect; the
    port loads tags by
    scanning the data
    directories, so it has
    no tag list to reorder.
  - Verification: read
    only; no code changed.

- **Item 225 — Attribute
  definitions, save/load:
  - The three 1.21.6-only
    attributes are gone
    from the definitions by
    construction: the
    catalog is data-driven
    (Catalog::from_json
    over the 1.21.5 export,
    checked against
    DATA_VERSION 4325), so
    camera_distance,
    waypoint_transmit_range
    and
    waypoint_receive_range
    simply are not in it.
    The one hardcoded
    registry-equivalent,
    the /attribute
    completion list in
    game/chat.rs, lost
    camera_distance (the
    waypoint pair was never
    in it).
  - AttributeInstance.load:
    a saved entry with no
    base now reads as 0.0
    (vanilla's
    getDoubleOr("base",
    0.0)) instead of being
    skipped — the item-126
    edge, fixed at the mob-
    load site. The horse
    jump-strength site
    needs no change: its <=
    0.0 default-back makes
    a missing base behave
    as 0.0 already. The
    save form (id, base,
    modifiers omitted when
    empty) was already the
    1.21.5 one.
  - Left for their owners:
    the waypoint tooltip
    arms and attribute-
    update events (items
    260/264) — string
    matches that never fire
    while the attributes do
    not exist. LivingCamera
    .camera_distance and
    the
    native_camera_distance
    mod hook stay: the live
    camera ignores the
    field and the hook is
    Horizon API surface,
    not vanilla behaviour.
  - Verification: cargo
    check -j 1 -p horizon-
    client --lib --tests
    0/0 (2m26s).

- **Item 226 —
  PathNavigation members:
  - Both 1.21.6-only
    members are already
    absent in the 1.21.5
    form.
    canNavigateGround: the
    port's
    can_navigate_ground is 
    GoalUtils.hasGroundPath
    Navigation (navigation
    instanceof
    GroundPathNavigation) —
    Ground and WallClimber
    only, Amphibious
    excluded (item 170) —
    not the 1.21.6 abstract
    method.
    setCanOpenDoors: no
    such method exists on
    the port's navigation;
    callers set the nav
    state's can_open_doors
    flag directly (bee,
    parrot, copper golem,
    zombie door-breaking,
    raiders when raided),
    which is what 1.21.5's
    evaluator-side setter
    does.
  - Verification: read
    only; no code changed.

- **Item 227 —
  test_instance POI type:
  - Removed the
    test_instance_block arm
    from save::poi_type:
    the
    minecraft:test_instance
    POI type (max tickets
    0, range 1) was added
    in 1.21.6 and does not
    exist in 1.21.5, so a
    test instance block is
    no longer a point of
    interest. The block
    itself, its beacon beam
    and the /test locate
    machinery stay — only
    the POI registration is
    gone.
  - Verification: cargo
    check -j 1 -p horizon-
    client --lib --tests
    0/0 (2m01s).

- **Item 228 —
  TrialSpawnerStateData:
  - 1.21.6 renamed
    TrialSpawnerData to
    TrialSpawnerStateData
    (and split its state
    into a Packed record);
    the serialized form is
    identical in both
    versions —
    registered_players,
    current_mobs,
    cooldown_ends_at,
    next_mob_spawns_at,
    total_mobs_spawned,
    spawn_data,
    ejecting_loot_table,
    same keys and defaults.
    The port already uses
    the 1.21.5 name (its
    docs cite
    TrialSpawnerData, the
    update tag cites TrialS
    pawnerData.getUpdateTag
    ) and its
    TrialSpawnerExt
    load/save read and
    write exactly those
    seven keys.
  - Verification: read
    only; no code changed.

- **Item 229 —
  LootTable.validate paths:
  - The diff is the
    validation-path
    spelling only: 1.21.6
    wraps the child path in
    a ProblemReporter.Index
    edFieldPathElement,
    1.21.5 passes the plain
    string ".pools[i]" /
    ".functions[i]" — the
    same path text either
    way, and diagnostics
    only. The port has no
    loot-table validation
    at all (item 206: only
    the partial predicate-
    graph check, which
    covers loot predicates,
    not tables), so neither
    spelling has a surface.
  - Verification: read
    only; no code changed.

- **Item 230 — Legacy chunk
  entity load:
  - The diff is the
    ValueInput refactor
    again (items 198/201):
    1.21.6 wraps the legacy
    entity load in a Proble
    mReporter.ScopedCollect
    or plus TagValueInput,
    1.21.5 calls EntityType
    .loadEntitiesRecursive
    on the raw tag list.
    The port has no
    ProblemReporter,
    TagValueInput or
    ValueInput anywhere;
    save::load_entities
    returns the raw entity
    tags, which is the
    1.21.5 form. The block-
    entity half
    (keepPacked,
    loadStatic) is
    identical in both
    versions.
  - Verification: read
    only; no code changed.

- **Item 231 — Server pack
  version accessor:
  - Pure accessor rename:
    1.21.6's packVersion(Pa
    ckType.SERVER_DATA) is
    1.21.5's getPackVersion
    (PackType.SERVER_DATA),
    same value, in the
    built-in vanilla
    datapack's metadata
    section. The port has
    no pack-version surface
    at all (the only
    mention is a doc
    comment in the port-
    pack upgrader), so the
    rename has nothing to
    land on.
  - Verification: read
    only; no code changed.

- **Item 232 — Empty vertex
  format:
  -
    DefaultVertexFormat.EMP
    TY (a format with no
    elements, for vertex-
    ID-driven draws) was
    added in 1.21.6. The
    port has no
    DefaultVertexFormat
    type at all — its
    vertex layouts are per-
    pipeline wgpu
    VertexBufferLayouts,
    and the only mentions
    of the Java formats are
    doc comments citing
    POSITION and BLOCK as
    shapes. The port's
    buffers: &[] pipelines
    are wgpu's own empty-
    layout idiom for
    fullscreen passes, not
    a port of the Java
    constant.
  - Verification: read
    only; no code changed.

- **Item 233 —
  addVertexWith2DPose:
  - 1.21.6 added a
    VertexConsumer default
    that transforms a 2D
    position by a
    Matrix3x2f and emits
    the vertex. The port
    has no VertexConsumer
    interface: its HUD
    batch applies the same
    JOML transformPosition
    math (m00*x + m10*y +
    m20, m01*x + m11*y +
    m21) to the batched
    vertices after the draw
    calls, in hud_layout.rs
    — identical arithmetic,
    applied at a different
    stage. Nothing version-
    specific to port.
  - Verification: read
    only; no code changed.

- **Item 234 —
  BiomeAmbientSoundsHandler
  :
  - Report false positive:
    the report flags one
    changed method, but the
    decompiled sources are
    byte-identical in both
    versions (5756 bytes
    each) and the per-item
    diff file is empty, so
    there is no
    1.21.6-to-1.21.5
    difference to port. The
    biome ambient sounds
    themselves are already
    the 1.21.5 form (item
    160).
  - Verification: read
    only; no code changed.

- **Item 235 — Subtitle
  overlay:
  - The 1.21.6
    nextStratum() call
    before the audible
    subtitles has no port
    surface: the stratum
    concept is gone from
    the port entirely (item
    212), and the subtitle
    overlay draws its rows
    in plain draw-call
    order — the 1.21.5
    form. The pose rename (
    pushMatrix/translate/sc
    ale 2D vs pushPose/tran
    slate/scale 3D with z
    0) is a no-op: the
    port's per-row pose
    carries the same
    translate (width -
    half_width - 2, height
    - 35 - row * 10) and
    scale 1.0.
  - Verification: read
    only; no code changed.

- **Item 236 — Model
  rotation angles:
  - ElementRotation::decode
    now rejects single-axis
    angles outside the
    1.21.5 five (0,
    +/-22.5, +/-45) with
    vanilla's message,
    instead of accepting
    any angle: 1.21.5's
    BlockElement
    deserializer throws on
    anything else, while
    1.21.6 widened the
    check to any angle
    within [-45, 45]. The
    port's EulerXyz branch
    (x/y/z keys, a non-
    vanilla extension) is
    untouched.
  - Verification: cargo
    test -j 1 -p horizon-
    assets --lib 53/53,
    incl. the new element_r
    otation_angles_are_the_
    1_21_5_five_only (10
    assertions).

- **Item 237 —
  RecipeButton:
  - The button draw matches
    1.21.5: the squeeze
    pose (1 + 0.1 sin(t/15
    pi) about x+8, y+12,
    item 177), the 25x25
    sprite blit, and the
    multi-recipe double
    draw — real item at
    offset +5/+5, fake item
    at +3/+3, same stack,
    same call order. The
    pose rename (pushMatrix
    vs pushPose, 2D vs 3D
    with z 0) and the
    pipeline swap (RenderPi
    pelines.GUI_TEXTURED vs
    RenderType::guiTextured
    ) have no surface.
  - Known gap: 1.21.5's
    renderItem passes z 10,
    so the real item covers
    the fake item where
    they overlap; the
    port's HUD batch has no
    depth (order decides),
    so the later fake-item
    draw covers the real
    one — the visible icon
    sits 2px up-left of
    vanilla on multi-recipe
    buttons with identical
    result displays.
    Swapping the emission
    order would break the
    fixture's call-order
    comparison.
  - Verification: read
    only; no code changed.

- **Item 238 —
  TextDisplayRenderer:
  - Report false positive,
    as item 234: the report
    flags one changed
    method in DisplayRender
    er$TextDisplayRenderer,
    but the decompiled
    sources are byte-
    identical in both
    versions (4383 bytes
    each) and the per-item
    diff file is empty, so
    there is no
    1.21.6-to-1.21.5
    difference to port.
  - Verification: read
    only; no code changed.

- **Item 239 — Recipe-book
  screen order:
  - Moved draw_carried
    before draw_recipe_book
    in the container draw:
    1.21.5's AbstractRecipe
    BookScreen.render calls
    super.render (which
    draws the carried item,
    AbstractContainerScreen
    .render) and only then
    the book, so the book
    covers a carried stack
    under the mouse. The
    port had it the other
    way round. The strata
    and the separate render
    CarriedItem/renderSnapb
    ackItem calls are
    1.21.6-only (the 1.21.6
    renderContents split);
    the port has no strata,
    and the snapback
    animation is still
    unimplemented (item
    171).
  - Verification: cargo
    check -j 1 -p horizon-
    client --lib --tests
    0/0 (2m36s).

- **Item 240 — Creative tab
  contents:
  - The four removals
    (dried ghast, all
    sixteen harnesses,
    music disc Tears, happy
    ghast spawn egg) are
    already absent by
    construction: the
    creative inventory
    loads from creative-
    inventory.json,
    exported at prepare
    time by VanillaCreative
    TabsExporter calling
    vanilla's own CreativeM
    odeTabs.tryRebuildTabCo
    ntents against the
    1.21.5 jar. No Rust
    code names any of the
    four items (item 174).
  - Verification: read
    only; no code changed.

- **Item 241 — Container
  background:
  - Pipeline swap only:
    1.21.6's RenderPipeline
    s.GUI_TEXTURED is
    1.21.5's RenderType::gu
    iTextured, over
    identical blit
    arguments. The port's
    generic-container
    background already
    draws the same two
    blits (top 176 x
    rows*18+17 from v 0,
    player half 176 x 96
    from v 126) in
    game/storage.rs.
  - Verification: read
    only; no code changed.

- **Item 242 — Options:
  - The four 1.21.6-only
    options have no port
    surface at all:
    cloudRange,
    keyQuickActions,
    musicFrequency and
    showNowPlayingToast
    (zero references; the
    only NowPlaying mention
    is the jukebox HUD
    overlay comment). Glint
    strength is applied at
    draw time (item 219),
    the 1.21.5
    setShaderGlintAlpha
    equivalent.
  -
    getFinalSoundSourceVolu
    me is only a name for
    the fold the port
    already does:
    final_source_volume
    documents that 1.21.5
    multiplies the source
    volume and applies the
    master as the listener
    gain — the same
    product. The options-
    file version line
    writes version:4325
    from DATA_VERSION, and
    the port parses
    key:value lines, so the
    LenientJsonParser vs
    JsonParser.parseReader
    change (both lenient in
    Gson) has no surface.
  - getCloudsType is
    already the 1.21.5
    form: clouds are off
    below render distance
    4, pinned by the settin
    gs_and_packed_alpha_gat
    e_clouds test.
  - Verification: read
    only; no code changed.

- **Item 243 — RopesLayer:
  - The layers directory's
    Only-in line is a
    sparse-tree artifact
    (the layers package
    exists in 1.21.5, with
    no changed classes
    extracted under it);
    the one class the diff
    actually carries,
    RopesLayer, is
    1.21.6-only by content:
    it renders the harness
    ropes on a leashed
    happy ghast wearing a
    harness. The port has
    no happy ghast and no
    harness (items
    144/174/224), so there
    is no surface.
  - Verification: read
    only; no code changed.

- **Item 244 —
  TrialSpawnerRenderer:
  - The same rename as item
    228, at the use site: T
    rialSpawnerStateData/ge
    tStateData() in 1.21.6
    is TrialSpawnerData/get
    Data() in 1.21.5. The
    rendered behaviour is
    identical (display
    entity from getOrCreate
    DisplayEntity, spun
    with
    getOSpin()/getSpin()).
    The port has no
    TrialSpawnerStateData
    anywhere; its trial-
    spawner render already
    uses getOrCreateDisplay
    Entity and the
    spin/o_spin fields.
  - Verification: read
    only; no code changed.

- **Item 245 — Leash render
  state:
  - 1.21.6's
    List<LeashState> (quad
    leashes) and its slack
    flag have no port
    surface: the mob model
    carries a single leash
    (leash: Option<Holder>,
    item 144) and
    leash_geometry pushes
    one Rope per leashed
    mob — the 1.21.5
    single-leash form. No
    LeashState type and no
    slack field exist
    anywhere in the port.
  - Verification: read
    only; no code changed.

- **Item 246 — Login player
  creation:
  - 1.21.6 constructs the
    ServerPlayer directly
    at login; 1.21.5 asks P
    layerList.getPlayerForL
    ogin, a factory hook
    whose vanilla default
    constructs exactly the
    same ServerPlayer —
    identical behaviour, an
    extension point only.
    The port has no player-
    construction hook: the
    guest record is built
    directly when the
    connection configures,
    and place_new_player
    places it. No surface.
  - Verification: read
    only; no code changed.

- **Item 247 — Raid
  validPlayer:
  - Pure accessor rename:
    for a ServerPlayer,
    level() and
    serverLevel() return
    the same ServerLevel,
    so validPlayer is
    identical in both
    versions. The port's
    raid code resolves
    players through the
    single level in scope
    (level_players,
    level.player_near) and
    carries alive on
    RaidPlayer — there is
    no level/serverLevel
    distinction to make.
  - Verification: read
    only; no code changed.

- **Item 248 —
  LootPool.validate paths:
  - The same validation-
    path spelling change as
    item 229, for loot
    pools: 1.21.6 wraps the
    child paths in
    ProblemReporter path
    elements, 1.21.5 passes
    plain strings
    (.condition[i],
    .functions[i],
    .entries[i], .rolls,
    .bonusRolls) —
    diagnostics only, and
    the two spellings
    differ in the
    singular/plural and
    camel/snake details
    only. The port has no
    loot-pool validation at
    all, so neither
    spelling has a surface.
  - Verification: read
    only; no code changed.

- **Item 249 — Pack version
  accessor:
  - The same accessor
    rename as item 231, at
    the pack-metadata read
    site: 1.21.6's
    packVersion(PackType)
    is 1.21.5's getPackVers
    ion(PackType), same
    value. The port has no
    pack-version surface
    (item 231), so the
    rename has nothing to
    land on.
  - Verification: read
    only; no code changed.

- **Item 250 — Happy ghast
  defaults:
  - The happy ghast's
    DefaultAttributes entry
    is 1.21.6-only (no
    happy ghast in 1.21.5).
    The port has no happy-
    ghast references at all
    (item 144) and its
    entity attribute
    defaults are data-
    driven from the 1.21.5
    export (item 225),
    which carries no such
    entry.
  - Verification: read
    only; no code changed.

- **Item 251 —
  StrictJsonParser:
  - The class is
    1.21.6-only and has no
    port equivalent (no
    StrictJsonParser type
    anywhere). Its 1.21.6
    call sites (SimpleJsonR
    esourceReloadListener,
    TagLoader,
    RegistryDataLoader,
    PlayerAdvancements,
    ServerStatsCounter,
    BlockStateModelLoader,
    ShaderManager,
    SpriteSourceList) use
    JsonParser.parseReader
    in 1.21.5 — Gson's
    lenient reader — while
    the port parses with
    strict serde_json
    throughout.
  - Known systematic gap
    (extending item 222):
    datapack, tag,
    advancement, stats,
    model and shader JSONs
    with comments, trailing
    commas or unquoted
    strings load in vanilla
    1.21.5 but are rejected
    here. The port has a
    lenient parser (java_te
    xt::parse_lenient), but
    it is wired in only for
    language files; routing
    every JSON loader
    through it is out of
    proportion for this
    item.
  - Verification: read
    only; no code changed.

- **Item 252 — Loot entry
  validate:
  - The same validation-
    path spelling change as
    items 229/248, for
    loot-pool entries:
    1.21.6 wraps the
    condition paths in
    ProblemReporter
    elements, 1.21.5 passes
    the plain string
    .condition[i] —
    diagnostics only. The
    port has no loot
    validation at all, so
    neither spelling has a
    surface.
  - Verification: read
    only; no code changed.

- **Item 253 — Client
  player spawn:
  - 1.21.5's Player
    constructor snaps the
    new player to the
    shared spawn (pos +
    0.5, y + 1, spawn
    angle); 1.21.6's
    constructor does not
    snap at all. The port
    already implements the
    1.21.5 behaviour at the
    join step: a player new
    to the world is placed
    by server_player_spawn
    (the PlayerSpawnFinder
    spread) with the world
    spawn's yaw and pitch,
    while a returning
    player loads their
    saved position over it.
    The constructor-
    signature difference
    has no Rust surface.
  - Verification: read
    only; no code changed.

- **Item 254 — jeb_ wool
  colour:
  - Done by item 211: Sheep
    RenderState.getWoolColo
    r is the other half of
    the ColorLerper change.
    The port's jeb_wool_rgb
    already implements the
    1.21.5 inline form
    exactly — cycle index
    floor(age)/25 + id,
    fraction (floor(age) %
    25 + frac(age))/25,
    per-channel lerp
    between consecutive dye
    wools — pinned by the
    10-assertion test from
    item 211.
  - Verification: read
    only; no code changed.

- **Item 255 — Client pack
  version:
  - The same accessor
    rename as items
    231/249, for the
    vanilla resource pack's
    metadata: 1.21.6's pack
    Version(PackType.CLIENT
    _RESOURCES) is 1.21.5's
    getPackVersion(PackType
    .CLIENT_RESOURCES),
    same value. The port
    has no pack-version
    surface (item 231), so
    the rename has nothing
    to land on.
  - Verification: read
    only; no code changed.

- **Item 256 —
  BufferBuilder cap:
  - 1.21.6 added a
    16,777,215-vertex cap
    to BufferBuilder
    (throwing past it);
    1.21.5 has no cap. The
    port has no vertex-
    count cap anywhere —
    its mesh and HUD
    buffers grow unbounded,
    the 1.21.5 form. (The
    16777215 hits in the
    tree are 0xFFFFFF
    colour values and
    shader depth packing,
    unrelated.)
  - Verification: read
    only; no code changed.

- **Item 257 — Patch value
  codec:
  - 1.21.6 decodes
    delimited component-
    patch values with
    registry context (regis
    tryFriendlyLengthPrefix
    ed); 1.21.5 uses the
    plain lengthPrefixed
    form. The wire form is
    identical (VarInt
    length + bytes) in
    both. The port's
    delimited read (item.rs
    read_stack) already
    takes the 1.21.5 form:
    each value is sliced
    out by its length and
    decoded through a fresh
    Reader over just those
    bytes, with no registry
    context passed.
  - Verification: read
    only; no code changed.

- **Item 258 — Dialog
  system:
  - The five dialog
    registry keys (DIALOG,
    DIALOG_TYPE,
    DIALOG_BODY_TYPE,
    INPUT_CONTROL_TYPE,
    DIALOG_ACTION_TYPE) are
    1.21.6-only. The
    minecraft:dialog
    registry left the
    loader in item 222;
    this item removes the
    rest of the system,
    already unreachable (no
    /dialog since item 145,
    no show_dialog/custom
    actions since item 175,
    no dialog packets).
  - Deleted: dialog.rs
    (model),
    dialog_screen.rs
    (layout),
    game/dialogs.rs (UI),
    2,702 lines;
    Game.dialogs state,
    every dialog_open gate,
    the close_dialogs
    calls, the
    send_custom_click no-op
    and drop_held_behind_di
    alog.
  - The DialogWarning
    screen is gone too:
    1.21.5's
    ServerLinksScreen has
    no warning button
    (verified in the
    decomp), so the title
    centres without the
    30px offset.
  - Protocol: the
    DialogHolder codec and
    dialog_tags sync are
    gone; UpdateTags is now
    ignored. Commands: the
    dialog codec dispatch,
    the show_dialog/custom
    click-action entries
    and the
    minecraft:dialog
    argument parser are
    gone. Datapacks: the
    dialog/ content kind is
    gone (its test reworked
    onto damage_type).
  - The shared 17px
    checkbox size (BOX)
    moved to
    widgets::CHECKBOX_BOX
    for the title and code-
    of-conduct screens. The
    pad Context's dialog
    flag became
    code_of_conduct (it
    also tracked that
    screen).
  - The dialog e2e checks
    left server_e2e.py and
    pad_menu_e2e.py; the
    command corpora keep
    their /dialog entries
    as negative tests (the
    waypoint precedent,
    item 145).
  - Verification: cargo
    check -j 1 -p horizon-
    client --lib 0/0
    (4m30s);
    protocol+command --all-
    targets 0/0 (22s);
    render --lib --tests
    0/0 (6m02s); client
    --bins 0/0; py_compile
    both e2e scripts. The
    client lib-test profile
    OOM-killed under load
    (known mode); the
    core_tests rework type-
    checks in CI.

- **Item 259 — Dialog
  bootstrap:
  - The built-in dialog
    registry bootstrap
    (Dialogs::bootstrap:
    server_links,
    quick_actions,
    custom_options) is
    1.21.6-only. The port
    has no built-in dialog
    entries anywhere — no
    quick_actions, no
    Dialogs bootstrap, no
    minecraft:server_links
    dialog id (the
    custom_options hits are
    legacy_custom_options
    world-gen settings,
    unrelated). The whole
    system is gone after
    item 258.
  - Verification: read
    only; no code changed.

- **Item 260 — ServerPlayer
  waypoint hooks removed**
  - ServerPlayer: the
    waypoint attribute
    hooks are gone.
    1.21.6's ServerPlayer.o
    nAttributeUpdated
    (receive range) and
    updatePlayerAttributes
    (crouch transmit
    modifier) have no
    1.21.5 counterpart, so
    the port's dispatch
    goes too: the receive-
    range arm in dispatch_a
    ttribute_updated, the
    WaypointRange arm of
    UpdatedAttribute and
    its UpdateWorld trait
    surface (level,
    is_server_level,
    waypoint_manager,
    attributes_field,
    attribute_value_in,
    track_waypoint), the
    waypoint_transmit_range
    kind mapping, and the
    MotionEvent::Waypoint
    variant with its probe
    arm. The crouch
    modifier was already
    absent. The Attributes
    map handle moves from
    UpdateWorld to
    LiveAttributeWorld,
    which is what refresh
    and value actually use.
    The two parity fixtures
    lose the waypoint impls
    (op 3 now falls through
    to Other); the harness
    drivers and oracles
    still send waypoint ops
    and call the 1.21.6
    Java APIs — out of
    subset, publish debt.
    The rest of the diff
    verifies: RootVehicle
    round-trips the full
    {Attach, Entity} form,
    the spectator camera
    ends when its target
    teleports, the spawn
    snap is item 253's, the
    recipe book is item
    223's, the
    level()/serverLevel()
    renames and the
    save/load refactor have
    no Rust surface, and
    openDialog and the
    happy-ghast stat are
    gone with items 258 and
    223. Known gap: the
    port never persists in-
    flight ender pearls
    (vanilla 1.21.5 saves
    and respawns them); the
    tooltip waypoint arms
    are item 264's.
  - Verification: cargo
    check -j 1 -p horizon-
    world -p horizon-client
    --lib 0/0; world_probe
    0/0; client_probe 0/0.

- **Item 261 — Server
  packet listener**
  - ServerGamePacketListene
    rImpl: every real delta
    is already the 1.21.5
    form. handlePlayerInput
    only records the keys
    (no idle-timer reset,
    no shift state — 1.21.6
    does both there); shift
    arrives via
    PlayerCommand actions
    0/1, the 1.21.5
    numbering 1.21.6
    dropped. Move
    validation is isPlayerC
    ollidingWithAnythingNew
    over plain
    getCollisions, not
    1.21.6's
    getPreMoveCollisions
    rename; the vehicle
    move checks the box
    deflated 0.0625 before
    and after, with no
    flying-vehicle
    exemption in the
    floating check. There
    is no change-game-mode
    packet in the protocol
    — the F3+F4 switcher
    sends /gamemode on a
    server.
    ChangeDifficulty
    silently ignores an
    under-permissioned
    guest (1.21.6 warns).
    Tag queries write NBT
    directly. The
    level()/serverLevel()
    renames, the
    send/connection.send
    refactor and the ValueO
    utput/ProblemReporter
    refactor have no Rust
    surface; movement
    recording has none
    either.
  - Verification: read
    only; no code changed.

- **Item 262 — JSON
  reload listener**
  - SimpleJsonResourceReloa
    dListener: the same
    ground as item 251.
    1.21.5 parses datapack
    JSON with Gson's
    lenient
    JsonParser.parseReader;
    1.21.6 uses
    StrictJsonParser. The
    port parses datapack
    JSON with strict
    serde_json
    (from_slice), so a
    datapack file with
    comments, trailing
    commas or unquoted
    strings loads in
    vanilla 1.21.5 and is
    rejected here — the
    systematic gap recorded
    in item 251, which
    already verified this
    class's call site.
    Routing the loaders
    through the lenient
    parser is out of
    proportion for this
    item.
  - Verification: read
    only; no code changed.

- **Item 263 — Sound
  events**
  - SoundEvents: sounds
    resolve by name from
    the loaded sounds.json
    (1.21.5 data), so the
    registry itself has no
    code surface — the call
    sites do. The dispenser
    no longer shears leads
    off (1.21.6's shearOffA
    llLeashConnections,
    with its SHEARS_SNIP);
    1.21.5's dispenser only
    tries the beehive and
    Shearable, and each
    species already plays
    its own shear sound.
    The lead-break sound is
    gone end to end: the
    sound flag left
    LeashBreak, break_leash
    and drop_leash, and
    lead_sound
    (item.lead.break) is
    deleted — 1.21.5 plays
    no sound when a lead
    breaks. The gliding-
    firework test pinned
    the 1.21.6 form
    (production was already
    1.21.5) and now pins
    leads kept and no lead
    sound. Verified: the
    knot sounds are the
    1.21.5 pair (leash_knot
    .break/place),
    sand.wind plays, dry-
    grass ambient is gone
    (item 160), and the
    ghastling/happy-ghast/h
    arness/unequip/tears
    sounds have no
    references. The
    protocol hash tables
    pin shears.snip only on
    the 26.3 side, where
    the adapter supplies
    it. Known gaps: the
    dried-ghast ambient
    code is inert (no block
    in the tables), and the
    port has no mob
    ambient-sound system at
    all, so the pufferfish
    ambient has nothing to
    land on.
  - Verification: cargo
    check -j 1 -p horizon-
    client --lib 0/0; --lib
    --tests 0/0.

- **Item 264 — Mob
  effects**
  - MobEffects: 1.21.6's
    invisibility adds a
    waypoint_transmit_range
    modifier (-1.0,
    ADD_MULTIPLIED_TOTAL);
    1.21.5's has none. The
    live effect-modifier
    table (effects.rs,
    applied on effect add)
    already has no
    invisibility arm; the
    stale one was the
    tooltip table, which
    showed a waypoint
    modifier on
    invisibility potions —
    removed, with both
    waypoint ranges from
    the tooltip's neutral-
    colour list. No
    waypoint_transmit_range
    or
    waypoint_receive_range
    reference remains in
    Rust code outside the
    version adapters,
    closing the waypoint
    removal (items 108,
    131, 145, 214, 225,
    260). The harness
    drivers still name the
    attributes
    (command_exec corpus,
    live attribute updates,
    native scene motion) —
    out of subset, publish
    debt.
  - Verification: cargo
    check -j 1 -p horizon-
    client --lib 0/0.

- **Item 265 — Move
  control**
  - MoveControl: 1.21.6
    added a public
    setWait(); 1.21.5 has
    no such method. The
    port has no set_wait
    either — its
    MoveControl.operation
    is a pub field, set
    directly where
    vanilla's own subclass
    controls set it
    internally (move
    completion, species
    stops). The only non-
    happy-ghast caller of
    setWait in 1.21.6 is Te
    mptGoal.ForNonPathfinde
    rs.stopNavigation,
    itself 1.21.6-only
    (1.21.5's TemptGoal has
    no stopNavigation); the
    port's tempt goal stops
    via nav_stop, the
    1.21.5 form. The
    Creeper and lingering-
    potion grep hits are
    setWaitTime, a
    different method.
  - Verification: read
    only; no code changed.

- **Item 266 — Block
  state properties**
  - BlockStateProperties:
    the DRIED_GHAST_HYDRATI
    ON_LEVELS property
    ("hydration", 0-3) is
    1.21.6-only. The port's
    dried-ghast code is
    inert on two layers:
    the block is not in the
    1.21.5 tables, so no
    state ever carries
    Class::DriedGhast and
    the tick dispatch never
    fires; and
    dried_ghast_tick
    returns early when the
    state table has no
    "hydration" key. Same
    standing as item 182,
    which left the enum
    entry and shapes arm
    inert. No
    "minecraft:dried_ghast"
    block name appears in
    the repo's static
    tables.
  - Verification: read
    only; no code changed.

- **Item 267 — Section
  storage**
  - SectionStorage: a pure
    accessor rename —
    1.21.6's
    dataVersion().version()
    vs 1.21.5's getDataVers
    ion().getVersion(), the
    same current data
    version in both spots
    (write and version
    check). No behaviour
    change; the port reads
    and writes DataVersion
    from its own pin
    (4325).
  - Verification: read
    only; no code changed.

- **Item 268 —
  Difficulty**
  - Difficulty: 1.21.6
    added a STREAM_CODEC
    (ByteBuf id mapper) and
    deprecated byId; the
    enum values and ids are
    identical in both
    versions. The port has
    no Difficulty codec
    type — it encodes
    difficulty as a plain
    i32 VarInt in both
    directions, the same
    wire form the codec
    spells. No behaviour
    change.
  - Verification: read
    only; no code changed.

- **Item 269 — Realms client**
  - Realms client: the port's
    1.21.6 Realms model is now
    the 1.21.5 one. Slots are
    Map<Integer,
    RealmsWorldOptions>
    directly: RealmSlot and
    RealmSetting are deleted,
    and hardcore is an options
    field read from the slot's
    settings array
    (parseSettings reads only
    that setting).
    RealmWorldOptions gained
    pvp, spawnMonsters,
    commandBlocks and hardcore,
    plus toJson, which writes
    only the non-default
    fields. updateConfiguration
    is gone: the settings tab
    saves through update, POST
    worlds/{id} with the name
    and description only;
    updateSlot posts
    options.toJson(). invite
    returns the whole
    RealmServer; join already
    returned the address. The
    preferredRegions endpoint,
    the region-preference DTOs
    and the RegionPreference
    screen are deleted.
    RealmsError lost its
    htmlPayload branch. The
    tier configuration and
    distance PUTs stay: they
    are Horizon extensions, in
    neither vanilla client.
    fetchFeatureFlags' MSA gate
    and
    shouldPlayButtonBeActive
    had no port surface. The
    LenientJsonParser
    difference is the item-251
    gap (strict serde_json).
  - Verification: cargo check -j 1 -p
    horizon-client --lib
    --tests: 0 errors, 0
    warnings. Tests reworked:
    slot parse (hardcore from
    settings), error bodies (no
    html case), description
    update, slot update posts
    options.toJson(); the
    preferred-regions test is
    deleted.

- **Item 270 — Book screens**
  - Book screens: the port's
    edit screen is now the
    1.21.5 form. saveChanges is
    gated on isModified (set by
    any page or title change
    and by appending a page),
    so closing an untouched
    book sends nothing. A page
    holds at most 1023
    characters (the filter is
    length < 1024, not <=). A
    click on the same page
    index within 250 ms selects
    the word there, and a third
    selects the whole page.
    Enter while signing signs
    whenever the title is non-
    empty (a title of spaces
    signs); only the button
    needs a non-blank title. A
    RunCommand link runs first
    and only then closes the
    book. Narration: the edit
    screen narrates nothing
    (NO_TITLE, no
    getNarrationMessage
    override), the signing mode
    narrates nothing, and the
    page buttons have the EMPTY
    message, so the 1.21.6 book
    .page_button.next/previous
    keys are gone. The in-
    screen signing, the 14-line
    limit, the cursor-to-end on
    page change (set_value) and
    the 300 ms blink were
    already the 1.21.5 form.
    Known gaps (version-
    neutral, not changed):
    Escape saves the book where
    vanilla discards it; the
    page has no drag selection;
    the cursor at the end of
    the text draws the 1 px bar
    where vanilla draws _.
  - Verification: cargo check -j 1 -p
    horizon-client --lib
    --tests: 0 errors, 0
    warnings (2m40s). The book_
    screens_narrate_nothing
    test pins the empty
    narration (run in CI; the
    test-profile build OOMs
    under load).

- **Item 271 — Post-processing chains**
  - Post chains: the port's
    1.21.6 model is now the
    1.21.5 form. Uniform blocks
    are gone: a pass names
    plain uniforms in a list,
    each {name, type, values?},
    set by setOnRenderPass's
    rule (one value fills a
    vector; an int type takes
    only the first; no values
    means zero — blur's Radius
    has none). Targets are
    fixed-size or fullscreen
    (the either codec); no
    persistent targets, no
    clear colour — every
    internal target is cleared
    to transparent black each
    frame. The GLSL translator
    wraps both stages' plain
    uniforms into one synthetic
    std140 block, and the
    vertex shader's in vec4
    Position becomes the
    fullscreen triangle's
    corner from the vertex
    index, so vanilla 1.21.5's
    #version 150 shaders
    translate (they could not
    before). Each frame the
    block holds ProjMat
    (setOrtho), OutSize, each
    <sampler>Size, then the
    config's uniforms, which
    win. The stand-ins name the
    1.21.5 shaders (post/blit,
    post/blur, post/sobel,
    post/screenquad; no
    core/screenquad) and the
    1.21.5 uniform names (InSca
    le/InOffset/InRotation,
    InverseAmount,
    ColorModulate,
    Resolution/MosaicSize,
    Red/Green/BlueMatrix,
    BlurDir/Radius,
    Scissor/Vignette); a config
    uniform the shader does not
    declare fails the pass, as
    createPass does. box_blur
    is round(Radius) with no
    menu-radius fallback (the
    fallback was the 1.21.6
    shader's rule); the menu
    blur is MenuBlur, not this
    chain. The Globals
    machinery stays (item 105's
    ground); Polytone post
    targets follow the same
    enum. Known gaps: strict
    serde_json for chain JSON
    (the item-251 gap — vanilla
    parses leniently); a config
    OutSize override is ignored
    by the stand-ins (they
    compute it); the
    Consumer<RenderPass> hook
    has no port surface (the
    port never runs the blur
    chain as a menu blur).
  - Verification: cargo check
    -j 1 -p horizon-client-
    render --lib --tests: 0
    errors, 0 warnings (2m03s).
    cargo test --lib -- post_:
    25 passed, 0 failed; the
    GPU tests early-return
    without a device on this
    VM, the pure
    config/pack/translator
    tests ran.

- **Item 272 — Eye of ender and respawn yaw**
  - Eye of ender and respawn
    yaw: a summoned eye
    (signalTo never ran) steers
    towards (0, 0, 0) in 1.21.5
    — there is no target to be
    null, tx/ty/tz default to 0
    — where the port flew it
    straight. The steering is
    now written as vanilla
    writes it (cos/sin of the
    angle), so an eye over its
    target's column steers east
    instead of dividing by
    zero. The trail particles
    now come from the server
    too (1.21.5 spawns them on
    both sides, with the
    steered motion); the
    clients draw their own as
    well, so each sees two, as
    in vanilla. signalTo
    already took the
    stronghold's integer block
    position (the 1.21.5 form);
    the tick's rotation has no
    surface (the eye is a
    camera-facing billboard),
    and the Item save/load was
    already the 1.21.5 form.
    Respawn yaw:
    TeleportTransition's world-
    spawn yaw is 0 in 1.21.5,
    not the shared spawn angle
    — the missing-respawn-block
    path (leaving the End,
    death respawn with no bed),
    the no-respawn-point path
    (createDefault), and the
    respawn-at-world-spawn
    teleport all face yaw 0
    now. An entity leaving the
    End through an end portal
    keeps its rotation (the yaw
    addend is 0); into the End
    it still faces west. A bed
    or anchor respawn still
    faces the block (look-at
    yaw), and a new player
    still joins facing the
    spawn angle (the
    ServerPlayer constructor,
    the same in both versions).
    Known gap: vanilla sets
    pitch 0 on every respawn
    transition; the port never
    touches pitch on respawn
    (pre-existing, same in both
    versions).
  - Verification: cargo check
    -j 1 -p horizon-client
    --lib --tests: 0 errors, 0
    warnings (5m58s). The
    end_portal_rotation test
    pins the unchanged out-of-
    End rotation (run in CI;
    the test-profile build OOMs
    under load).

- **Item 273 — Desert ambient sounds**
  - Desert ambient sounds:
    already the 1.21.5 form
    (item 160).
    playAmbientBlockSounds: the
    block in #plays_ambient_des
    ert_block_sounds under open
    sky; idle 1/1600 and wind
    1/10000 in desert or
    badlands, each only when
    three of the four surface
    columns 8 away carry a tag
    block; the draws run idle-
    then-wind and
    shouldPlayAmbientSound is
    re-evaluated per branch.
    Dead bush: 1/150, silenced
    4 in 5 on red sand or
    terracotta, sounding only
    when the two blocks below
    are both in the tag. Sand
    and terracotta tick the
    sounds; short and tall dry
    grass have no animateTick
    (Animate::None). No
    1.21.6-only tag
    (TRIGGERS_AMBIENT_DESERT_*)
    or chance (2100/200/130)
    remains anywhere.
  - Verification: read only; no
    code changed.

- **Item 274 — Sky renderer**
  - SkyRenderer: the diff is
    the rendering-API refactor
    between the versions
    (DynamicTransforms uniform
    blocks vs setShaderColor,
    buffer-creation signatures,
    GpuTextureView vs
    GpuTexture, drawIndexed
    arity) — none of it has a
    Rust surface; the port is
    wgpu-native. Every drawn
    value is the same in both
    versions: the sky disc's
    colour, the dark disc's
    black with the +12
    translate, the star
    brightness, the End sky's
    white, the star seed 10842.
    The one behavioural point —
    1.21.5 disables fog for the
    stars and restores it after
    — is already the port's
    form: stars.wgsl takes no
    fog uniform and computes no
    fog. The End sky's
    setFilter(TriState.FALSE)
    is the item-158 inert case
    (the port's own texture
    loading).
  - Verification: read only; no
    code changed.

- **Item 275 — Datapack commands**
  - Datapack commands:
    /datapack create is
    1.21.6-only, so the tree
    nodes (create + id +
    description), the
    create_pack executor (pack
    folder, data folder,
    pack.mcmeta) and its
    is_portable_name helper are
    deleted; /datapack
    list/enable/disable are
    unchanged (enable keeps
    first/last/before/after,
    level 2).
    FileUtil.isPathPartPortable
    was create_pack-only — the
    1.21.5 form inlines the
    reserved-Windows-names
    matcher at both call sites,
    same behaviour. /function
    and /reload keep level 2
    (hasPermission vs the
    lambda is the same
    predicate).
  - Verification: cargo check
    -j 1 -p horizon-command -p
    horizon-client --lib
    --tests: 0 errors, 0
    warnings (1m48s).

- **Item 276 — HangingEntity**
  - HangingEntity: already the
    1.21.5 form. Direction is a
    plain field
    (Painting/ItemFrame facing,
    u8), rides the add-entity
    packet's data int, no
    DATA_DIRECTION; frame
    item/rotation at 8/9 and
    the painting variant at 8
    are the 1.21.5 indices.
    Spawn yaw/pitch follow
    setDirection (yaw 2D*90;
    frame pitch +-90 vertical).
    rotate()'s yaw-base
    difference has no surface:
    the hanging NBT readers
    discard Rotation, and save
    writes yaw from facing. Its
    bbox difference is moot:
    placeEntities' snapTo
    rebuilds the box from the
    0.5x0.5 type dimensions in
    both versions
    (reapplyPosition -> setPos
    -> makeBoundingBox). Known
    gaps, version-neutral:
    turn_facing applies mirror-
    then-rotate while vanilla's
    entity path applies rotate-
    then-mirror (javap:
    Mirror.getRotation is 180
    on the mirrored axis, else
    NONE; getClockWise
    N->E->S->W), so a north-
    facing painting under
    mirror + 90-degree rotation
    faces W in the port, E in
    vanilla; vanilla's own
    entity path disagrees with
    its block path here.
    place_entities yaw uses the
    base Entity.rotate arms,
    not the hanging override's
    swapped arms -
    unobservable, since hanging
    yaw is discarded at spawn.
  - Verification: read only; no
    code changed.

- **Item 277 — Tab bar and statistics screen**
  - Tab bar and statistics
    screen: verified, no code.
    StatsScreen has no tab bar
    in either version - plain
    footer buttons, Items/Mobs
    disabled when their lists
    are empty (initButtons
    identical in both); the
    port's MenuTabBar gating
    (empty tabs unchoosable,
    the cycle skips them, a
    digit naming one is taken
    but switches nothing)
    mirrors that shared
    disabled-button rule, not
    the 1.21.6 TabNavigationBar
    API. isPauseScreen is
    !isLoading in 1.21.5,
    always true in 1.21.6 - but
    the port has no loading
    state (stats load
    synchronously locally,
    arrive via AwardStats
    remotely with no loading
    UI), so the conditional
    collapses to always true
    and Screen::Stats => true
    stands. Create-world tab
    cycling is already the
    1.21.5 form (plain
    floorMod, no active skip).
    The Realms config tab bar's
    active-state skip and
    onSelected / onDeselected
    (settings save and draft
    refill) are the 1.21.6
    tabbed config screen - the
    structural gap item 11
    documented and deferred to
    item 404, not this item's.
    GridLayoutTab's 1.21.6-only
    getTabExtraNarration
    returns empty; no narration
    difference. The
    RenderPipelines /
    setTooltipForNextFrame
    renames have no surface.
  - Verification: read only; no
    code changed.

- **Item 278 — StructureBlockEntity**
  - StructureBlockEntity:
    already the 1.21.5 form.
    save/load carry the full
    field list (name, author,
    metadata, pos, size,
    rotation, mirror, mode,
    ignoreEntities, strict,
    powered, showair,
    showboundingbox, integrity,
    seed) with the 1.21.5
    clamps (pos +-48, size
    0..48) and defaults (pos
    [0,1,0], mode Data,
    ignoreEntities true,
    showboundingbox true,
    integrity 1.0).
    template_from_world skips
    only structure voids - the
    1.21.6 ignoreBlocks list is
    already gone. level() vs
    getCommandSenderWorld() is
    a pure rename.
  - Verification: read only; no
    code changed.

- **Item 279 — Enchanting and station screens**
  - Enchanting, bundle tooltip,
    stonecutter, brewing,
    grindstone: verified, no
    code. The 1.21.6-only
    containerTick line (pinning
    experienceDisplayStartTick)
    has no surface - the port
    has no such concept and its
    XP bar has no time-based
    flash. The enchanting book
    is already the 1.21.5
    manual-pose form (translate
    (19,17) in the 38x31 box,
    scale 40, X 25 degrees, the
    (1-open) offset, Y
    -(1-open)*90-90, X 180;
    page flips
    clamp(frac(flip+0.25/0.75)
    *1.6-0.3)). Offer colours
    keep the opaque spellings
    with identical RGB (the
    established alpha-0
    pattern). The bundle
    tooltip draws back sprite,
    item, front highlight in
    the same order in both
    versions; its empty text is
    0xFFAAAAAA. Stonecutter,
    brewing, grindstone and the
    cycling slot background
    differ only by the
    RenderPipelines / tooltip
    renames, same geometry and
    sprites.
  - Verification: read only; no
    code changed.

- **Item 280 — Armadillo, bat, frog models**
  - Armadillo, bat, frog
    models: already the 1.21.5
    form, the same refactor as
    items 137/146/157. The port
    looks animations up per
    call by name from the
    animations map (run =
    Model.animate, walk  =
    animateWalk) - no baked
    KeyframeAnimation exists;
    keyframes ::Animation is
    pure definition data
    (length, looping, bones by
    name). Armadillo walks at
    (16.5, 2.5) with roll-out,
    roll-up and peek runs; bat
    runs BAT_FLYING and
    BAT_RESTING; frog runs
    jump, croak and tongue,
    swims at (1.0, 2.5) or
    walks at (1.5, 2.5), runs
    the idle-water animation,
    and shows croaking_body
    only while the croak state
    runs. The definitions
    themselves are unchanged
    between versions.
  - Verification: read only; no
    code changed.

- **Item 281 — GsonHelper**
  - GsonHelper: verified, no
    code - the same ground as
    item 251, confirmed from
    this side. 1.21.5's
    fromJson / fromNullableJson
    / parse take a lenient flag
    and default to strict (the
    no-boolean overloads pass
    false), while 1.21.6 is
    always strict; the port's
    strict-everywhere default
    therefore matches 1.21.5's
    default. What it misses is
    only the explicit lenient
    call sites: 1.21.5's
    datapack, tag, advancement,
    stats, model and shader
    loaders read through Gson's
    lenient
    JsonParser.parseReader (Sim
    pleJsonResourceReloadListen
    er confirmed), while the
    port parses those with
    strict serde_json. The port
    HAS lenient parsers
    (java_text::parse_lenient, 
    datafix::json::parse_lenien
    t) and strict ones
    (parse_strict,
    parse_prefix_strict), wired
    in only for language files
    and option values; routing
    every JSON loader through
    lenient - each site needing
    a Json to serde_json
    conversion - stays out of
    proportion, as item 251
    judged.
  - Verification: read only; no
    code changed.

- **Item 282 — GoalUtils and DoorInteractGoal**
  - GoalUtils and
    DoorInteractGoal: already
    the 1.21.5 form.
    can_navigate_ground is
    Ground + WallClimber,
    exactly 1.21.5's instanceof
    GroundPathNavigation
    (javap:
    WallClimberNavigation
    extends
    GroundPathNavigation;
    AmphibiousPathNavigation
    extends PathNavigation
    directly) - item 170's
    ground. mobRestricted /
    isRestricted match the
    1.21.5 formulas (centre
    +0.5, radius + h + 1.0,
    strict <) in
    Mob::restricted and
    within_home; the
    home/restriction naming is
    a pure rename.
    DoorInteractGoal.canUse
    matches 1.21.5 (ground +
    horizontal-collision guard,
    path nodes to min(next+2,
    count), door at y+1,
    distance squared <= 2.25,
    head-block fallback); the
    1.21.5 cast is safe behind
    the guard, so the 1.21.6
    cast-free form behaves
    identically.
  - Verification: read only; no
    code changed.

- **Item 283 — SectionRenderDispatcher**
  - SectionRenderDispatcher
    (outer) and RebuildTask:
    verified, no code - the
    same ground as items 57 and
    176, extended to these
    classes. The queue, upload
    and buffer machinery
    (toUpload, toBatchCount,
    SectionBuffers, per-layer
    GPU upload) is GPU
    architecture with no port
    surface; the port meshes
    its own way (Job::Mesh) and
    uploads synchronously. The
    debug stats line renders pC
    from streaming.pending()
    and aB 0 (no buffer pool),
    in vanilla's format; pU is
    omitted, version-neutral,
    since the port has no
    pending-upload queue.
    RebuildTask's null-region
    to EMPTY has no surface -
    the port never meshes
    unloaded regions (lit-
    columns gate) and continues
    missing neighbours as
    Source::Missing (item 57's
    documented divergence).
    updateGlobalBlockEntities
    has no surface (no per-
    section global block-entity
    tracking, items 25/57/135).
    hasAllNeighbors, the
    translucency resort and
    empty-section visibility
    are items 57 / 176's
    ground; the nested
    CompiledSection,
    SectionBuffers and
    TranslucencyPointOfView
    classes are item 403's
    (rank 4).
  - Verification: read only; no
    code changed.

- **Item 284 — ResourceOrIdArgument**
  - ResourceOrIdArgument,
    NestedLootTable,
    CompositeLootItemCondition:
    code - resource_or_id
    rewritten to the 1.21.5
    parseInlineOrId form: the
    inline SNBT value first,
    then the id; an inline
    string is a reference
    decoded by the holder
    codec, so an unknown id
    reports failed_to_parse
    with 'Failed to get element
    ... from registry ...',
    never the 1.21.6-only
    no_such_element (also fixed
    at the provider site in
    compute.rs). Input that is
    neither a whole value nor a
    whole id is invalid; an id
    read that finds nothing is
    argument.id.invalid. The
    lookup stays strict with an
    empty table - the slot path
    relies on unknown ids
    erroring. The validation-
    path spellings in
    NestedLootTable and
    CompositeLootItemCondition
    have no surface (no loot
    validation, items
    229/248/252). Suggestions
    already match: /loot and
    /item modify attach their
    own in 1.21.5.
  - Verification: cargo test -j
    1 -p horizon-command --lib:
    54 passed; cargo check -j 1
    -p horizon-client --lib:
    clean.

- **Item 285 — GameTest framework**
  - GameTest framework: code -
    prepare_test_structure is
    now the 1.21.5 direct form
    (returns (), the block
    entity required, never
    null) and both callers
    dropped their null checks:
    the in-place spawner
    prepares then starts
    execution, the grid spawner
    prepares then advances the
    grid unconditionally. The
    port inserts a default test
    instance block when the
    block has none, so prepare
    never failed anyway - the
    bool was dead.
    makeMockPlayer and
    assertBlockTag have no
    surface (no mock player, no
    such assertion in the
    port's helper).
  - Verification: cargo check
    -j 1 -p horizon-client
    --lib --tests: clean.

- **Item 286 — GhastModel and Ghast**
  - GhastModel and Ghast: code
    - the 1.21.5 forms were
    already ported (move
    control's stepped-box
    canReach with fixed 0.1
    acceleration, the single-
    draw random float, the look
    goal, the tentacle formula
    0.2*sin(age*0.3+i)+0.4 over
    tentacle0..8,
    ExplosionPower byte default
    1, no quad leash or 10/16
    distances, no FLYING_SPEED
    read). One gap fixed:
    ghasts and phantoms now
    skip fall damage
    (FlyingMob's no-op in
    1.21.5, their own overrides
    in 1.21.6 - the same either
    way); the port had exempted
    only bats and bees. The
    model renames
    (PartNames.tentacle vs
    createTentacleName,
    animateTentacles inline vs
    static) produce identical
    names and poses.
  - Verification: cargo check
    -j 1 -p horizon-client
    --lib --tests: clean.

- **Item 287 — RenderType**
  - RenderType: verified, no
    code - the diff is the
    rendering-API refactor
    between the versions
    (buffer sizes, the
    DynamicTransforms uniform
    block, GpuTextureView vs
    GpuTexture, drawIndexed
    arity, the GUI render types
    moving out of RenderType),
    none of which has a Rust
    surface; the port is wgpu-
    native with its own HUD
    batch and pipelines. The
    one structural point
    already matches 1.21.5: the
    port's layer enum carries
    Translucent (with Solid,
    Cutout, CutoutMipped,
    Tripwire), the chunk-layer
    set 1.21.6 dropped. The
    TextureStateShard TriState
    blur flag is inert (item
    158's ground), and debug
    lines are the port's own
    triangle-strip shaders, not
    a RenderType factory.
  - Verification: read only; no
    code changed.

- **Item 288 — ThrownTrident**
  - ThrownTrident: verified, no
    code - the diff is only the
    save-API refactor
    (ValueInput/ValueOutput vs
    CompoundTag); the
    serialized form is
    identical in both versions
    and the port already writes
    it: DealtDamage as a byte,
    read back with default
    false, and loyalty re-
    derived from the loaded
    item's Loyalty enchantment
    on load (trident.rs:111).
  - Verification: read only; no
    code changed.

- **Item 289 — Ban/save/stop commands**
  - Ban, save and stop
    commands: verified, no code
    - the only delta is the
    permission predicate's
    spelling
    (Commands.hasPermission(N)
    vs a lambda calling
    hasPermission(N)), the same
    predicate in both versions.
    The levels match: ban, ban-
    ip, banlist and transfer at
    3 (ADMINS), save-off, save-
    on and stop at 4 (OWNERS).
  - Verification: read only; no
    code changed.

- **Item 290 — Interaction**
  - Interaction: verified, no
    code - the diff is only the
    save-API refactor; the
    serialized form is
    identical in both versions
    and the port already writes
    it (InteractionState:
    width/height floats default
    1.0, response bool default
    false, attack/interaction
    as optional PlayerAction
    records with player int
    array and timestamp long,
    omitted when unset).
  - Verification: read only; no
    code changed.

## What remains, by cause

### Behaviour work from the diff (in progress — rank 1 done)

The 426 ranked items are the port's work list; **290 of 426 are
closed** (rank 1 complete, rank 2 complete, rank 3 at 22/133). The work is the dialog/waypoint/happy-ghast removals'
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
