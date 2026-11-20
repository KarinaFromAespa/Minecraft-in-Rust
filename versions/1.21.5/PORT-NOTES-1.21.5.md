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

## What remains, by cause

### Behaviour work from the diff (in progress — rank 1 done)

The 426 ranked items are the port's work list; **9 of 426 are
closed** (rank 1 complete). The work is the dialog/waypoint/happy-ghast removals'
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
