# Port notes: Minecraft 1.21.3

1.21.3 is the eighth adjacent delta port in the **Components era
(E6, 1.20.5–1.21.11)**, taken from 1.21.4. It is a cross-protocol
delta: protocol **769 → 768**, world version 4189 → 4082, data
pack 61.0 → 57.0 and resource pack 46.0 → 42.0 — and the packet
set changes on the serverbound side: the single `pick_item`
packet comes back in place of the split `pick_item_from_block` /
`pick_item_from_entity` pair, `player_loaded` is removed, and 25
play serverbound packets are renumbered, so this port needed a
packet rewire plus the adapter re-base. The measured delta is
**313 ranked work items**. The port is the mechanical layer: the
pin is set, the six generated tables are regenerated against
1.21.3's own jar, the adapter ladder is re-based to the new
protocol, the component codec is re-pinned (the pin's
`custom_model_data` is an int again), the verification gates
pass, and the remaining work is sized, ranked and documented.
The behaviour code in this tree is still the 26.3 base's,
inherited through 1.21.11, 1.21.10, 1.21.9, 1.21.8, 1.21.7,
1.21.6, 1.21.5 and 1.21.4; the port is **in progress**, not
parity-verified.

## Provenance

- Base: the programme's completed 1.21.4 tree — the `port/1.21.4`
  branch at 058ad0a, which contains the 1.21.4 tree including the
  1.21.8 code changes (packet rewire, adapter re-base, oracle port),
  the jsr305 jar, the both-forms `gen_version_consts.py`, the
  jsr305 classpath fix in `tools/port_regen.py`, and the full
  ported oracle. No overlay was needed. Published 1.21.4:
  main@58535d82.
- 1.21.3's facts, from its own version JSON and generated tables:
  protocol **768**, world (data) version **4082**, data pack
  format **57.0**, resource pack format **42.0**, Java **21**,
  stable release.
- Prepare ran clean from the Mojang manifest (2026-11-19):
  **8,726 classes indexed, 70,951 methods**; mappings: classes
  **8,726 mapped / 8,726 total**, methods **77,484 mapped /
  77,484 total (58,937 renamed)**, fields **39,707 mapped /
  39,707 total (39,461 renamed)**, 0 placeholders; named jar
  32,659,897 bytes; datagen ran. Nothing Mojang-derived is
  committed in this tree.
- Diff 1.21.4 → 1.21.3: **313 ranked work items** (rank 1: 9,
  rank 2: 189, rank 3: 90, rank 4: 23, rank 5: 2). Classes +89
  −220, 41 moved, **773 changed (3,194 methods); 656 of the
  changed or removed ones are ported or proved by Horizon**.
  Data files +112 −94 ~1,380; assets +862 −1,552 ~152; en_us
  keys +1 −29 ~6.

## The delta: pick-item reunified, resin and eyeblossoms go away

Rank 1 is the version pin (`version.rs`), the six regenerate
items, the save-format item (DataFixers 4189 → 4082: 0 schemas,
2 fixes added, 4 datafix classes changed —
`EntityPaintingFieldsRenameFix`,
`ItemStackComponentRemainderFix`) and the version facts
(protocol 769 → 768, world 4189 → 4082). Rank 2 is 189 harness
reruns. Rank 3 is 90 unproved items. Rank 4 is 23 items. Rank 5
is 2 items. The only command removed is `attribute target
attribute base reset` (a sub-node — see "Tables" below).

The visible content changes are the removals of 1.21.4 features:

- **Resin and eyeblossoms**: resin clump/block/bricks/stairs/
  slab/wall/chiseled bricks (blocks and items) and the open/
  closed eyeblossom (blocks, items, potted forms) — 11 blocks,
  10 items — plus the `pale_oak_leaves` particle and 15 sound
  events (5 eyeblossom, 9 resin, `entity.creaking.twitch`).
- **The creaking transient is back**: 1.21.3 has the
  `creaking_transient` entity (30) that 1.21.4+ lack — the only
  registry that *grows* going down (entity +1). The creaking
  heart goes back to 9 states (axis × disabled/dormant/active,
  axis-major) from 1.21.4's 12 (boolean `active` × axis ×
  natural).
- **Three 1.21.4 packet layouts revert**: `LevelParticles` has
  no `alwaysShow` field, `SetHeldSlot` carries a byte (not a
  VarInt), and `PlayerInfoUpdate` has no `UPDATE_HAT` action
  (7 actions, not 8) — verified with `javap -c` against the
  1.21.3 named jar. See "Packet rewire" below.
- **`custom_model_data` is an int again**: the four-list form
  (floats/flags/strings/colors) arrived in 1.21.4; 1.21.3's
  component is a single VarInt. See "Component re-pin" below.

The `data_component_type` registry is **unchanged** (0 removed,
0 added) — the component constants carry over from 1.21.4.

The id shift rules across the delta: items <226 unchanged;
228–375 shift −2; 383–1166 shift −9; >1167 shift −10. Blocks
<333 unchanged; 334–339 shift −1; 346–1090 shift −7; >1094
shift −11. Entities <30 unchanged; ≥30 shift **+1** (the
transient gap). Sounds <384 unchanged; 385–522 shift −1;
528–1327 shift −6; >1336 shift −15. Particles <34 unchanged;
>34 shift −1. That is why the adapter re-base and the test
re-pins below are mechanical but wide.

## Tables: 5 regenerated, 1 verified unchanged

| Generator | Result vs 1.21.4 |
|---|---|
| `version_consts` | regenerated (+8 −8 — the pin: 1.21.3 / 768 / 4082 / 57.0 / 42.0) |
| `packet_ids` | regenerated (+28 −32 — 3 packets removed, 1 added, 25 renumbered) |
| `entity_tracking` | regenerated (+1 −0) |
| `lan_command_order` | regenerated (+1 −1) |
| `command_suggestions` | regenerated (+0 −8) |
| `vanilla_command_order` | unchanged (verified — see below) |

The regen took two runs (commit 431c03f7c3 on `port/1.21.3`,
then a clean re-run). Run 1 regenerated the first five tables
and failed `vanilla_command_order` because `horizon-protocol`
would not compile until the packet rewire landed. Run 2
regenerated nothing: all six tables were byte-identical,
including `vanilla_command_order` — the generator did run
(it wrote 28 nodes), but `vanilla_order.rs` lists only
top-level command nodes while the removed `attribute target
attribute base reset` is a sub-node, so the output is
unchanged. The committed `regen.json` lists all six
generators with their run-1–2 statuses combined (D1
completeness, per the gate change at main@15ff93a8). A fresh
regen reproduces all six committed tables byte-for-byte —
see `regen-determinism.json`, no overrides.

## Packet rewire: pick-item reunified, player_loaded removed

Commit b78c853 on `port/1.21.3` removes the three 1.21.4
packets (`PickItemFromBlock`, `PickItemFromEntity`,
`PlayerLoaded`) and adds `PickItem { slot: i32 => VarInt }`
(a single varint slot — verified with `javap -c` against the
1.21.3 named jar). The `PlayerLoaded` sends in
`horizon-protocol/examples/probe.rs` went with it (1.21.3 has
no `player_loaded` packet).

Commit 2dc3a62 reworks the game-side wire boundary around the
missing packets, following vanilla 1.21.3's own flows
(verified with `javap -c`):

- **Pick resolution moves to the client.** Vanilla 1.21.3's
  client resolves the pick stack itself (`getCloneItemStack` /
  `getPickResult`, `addCustomNbtData`, `findSlotMatchingItem`):
  in creative it sets the picked item and syncs the slot; in
  survival a hotbar match selects locally and sends nothing,
  and only a main-inventory match sends
  `ServerboundPickItemPacket(slot)`. `remote.rs` `pick()` now
  does exactly that, and the LAN guest arm sends the slot
  through `pick_item::pick_slot` (out-of-range slots ignored),
  with the server echoing the held slot back.
- **Guests are loaded from join.** Vanilla 1.21.3 has no load
  gating at all — `ServerPlayer` and
  `ServerGamePacketListenerImpl` have no load/timeout methods —
  so the `loaded`/`loading`/`load_timeout` guest defaults are
  now loaded-from-join, the hub no longer ticks a load timeout
  or fires `HostEvent::PlayerLoaded` on expiry, and respawn no
  longer unloads the client. `HostEvent::PlayerLoaded` is kept
  but never fires at this pin; `CLIENT_LOADED_TIMEOUT` is kept,
  unused, for the 1.21.4 shape.
- **The native pick-item hook has no call site at this pin.**
  `native_player_pick_item` is kept with `#[allow(dead_code)]`
  because `crates/horizon-client/build.rs` generates the native
  mod API from the hook signatures and fails without it; the
  server never learns what the guest picked, so nothing calls
  it. It returns with the pick packets in 1.21.4.
- `client_session.rs` drops the 1.21.4 tree's pick-item →
  container-click conversion (the `held` field, `hold()`,
  `pick_item()`, the `SET_CARRIED_ITEM`/`SET_HELD_SLOT`
  tracking): the pin has `pick_item`, so a 1.21.3 pick passes
  through unchanged.

**The three 1.21.4 layout reversions** (commit c23dc1a):
`LevelParticles` loses `always_show`, `SetHeldSlot` becomes a
byte, and `PlayerInfo` loses `show_hat` (hats always show at
this pin — the tab-list default stands). The translate layer
carries the conversions: `pin_layout` strips the newer layouts
to the pin wire (a `strip_hat` helper re-emits player-info
without the hat bool), `world.rs` `down()` converts the pin
wire back up for 26.2 clients, and the `v1_21_3` module keeps
its three 1.21.3→1.21.4 UP arms (its `PLAYER_INFO_UPDATE` arm
now writes the 1.21.4 form via a `write_with_hat` helper).

## Adapter re-base to 768

The adapter ladder (`crates/horizon-protocol/src/adapter/`) is
re-based from the 769 pin to 768 (commit c3f000f on
`port/1.21.3`; scripts `rebase768.py` and `fix_slot_display768.py`
in the Stream 1 workdir, modelled on the 1.21.4 `rebase769.py`).
The rebase was recovered from diffing the regenerated tables
against the 769 tree:

- INHERIT map: 1.21.1→STORE/1.21.3, 1.21.4→PIN, 1.21.5→PIN,
  1.21.6→PIN, 1.21.8→PIN.
- `REGISTRY_REASSIGN` is now **empty**: the pin *has*
  `creaking_transient`, so no reassignment is needed. The
  renames are unchanged from 769 (`splash_potion→potion` for
  1.21.5+, `iron_chain→chain`, `team_color→color`).
- Creaking-heart state aliases flipped direction (new → pin):
  1.21.4's 12 states and 1.21.5+'s 18 states both map to the
  pin's 9 states, natural=false listed first so DOWN prefers
  the placed heart. The **transient DOWN hand-patch** maps pin
  30 (`creaking_transient`) to `creaking` in all 8 newer
  modules (v1_21_4/5/6/8 → 29, v1_21_10 → 30, v1_21_11/26_1_2/
  26_2 → 31).
- **Serializers unchanged**: `EntityDataSerializers` is not in
  the diff's changed classes and the 1.21.3 jar has the same 31
  registerSerializer calls in the same order — all remap values
  carry over, only comments moved.
- `adapter.rs` pin swap: V1_21_3 is now the pin identity static
  (quirks inline: `wolf_variant_holder`,
  `components_before_1_21_5`, `components_before_1_21_6`,
  `integer_custom_model_data`, `trail_without_duration`);
  V1_21_4 chains from V1_21_3.quirks with the two 1.21.3 quirks
  switched off; V1_21_1 chains from V1_21_3.quirks unchanged.
  `Quirks::PINNED` now says "the pinned 1.21.3's own wire
  layouts" with both component quirks true.
- SlotDisplay: the `slot_display` registry is identical in
  1.21.3, 1.21.4 and 1.21.5 (8 entries, same order), so the 770
  fix values apply unchanged; the v1_20_3 ENTITY_TYPE_DOWN
  breeze anchor re-applied with the 768 anchor (`(31, 20, 1)`
  — pin 30 is the transient gap).

## Component re-pin: custom_model_data is an int again

The `data_component_type` registry is unchanged, so the
component constants carry over — but two pin-codec behaviours
change with the pin (commit 2d68552):

- **`Component::write`'s CustomModelData arm** now uses
  `ids::quirks_written().integer_custom_model_data` (DOWN
  quirks when writing for an older version, else PINNED). It
  previously wrote the int form only under an active DOWN
  session, so a pin-wire `DelimitedSlot` round-trip wrote the
  list form the pin reader parses as an int — the list count
  came back as the value.
- **The legacy saved-form reader** (`v1_20_3::from_tag`, kind
  14) accepts a plain-int `CustomModelData`: at this pin the
  datafixer leaves the int, and the list form arrived in
  1.21.4. The write side already wrote the first float as an
  int.
- **Particle codec re-pinned** (real code, `entity_data.rs`):
  trail 46, vibration 45, item 44, item_slime 47, shriek 100,
  instant_effect 43 — and the trail *write* now skips the
  duration when `trail_without_duration` (the read already
  did; the write unconditionally wrote it — a pre-existing
  DOWN gap, now a pin-correctness fix).
- **The tool/can_break internal forms** are unchanged from
  1.21.4: the pin codec's skip reads the tool's
  `canDestroyBlocksInCreative` bool (the internal form the
  legacy rewrite appends on UP — the bool itself arrived in
  1.21.5, verified with `javap`) and the can_break predicate's
  two matcher varints. The fixtures carry those bytes; the
  wire forms do not.

The registry shifts forced wide test re-pins: diamond 836→827,
brick 954→945, calcite state 23329→22785, filled map 1031→1022,
tadpole 123→124, turtle 130→131, zombie villager 145→146, player
kind 147→148, angler sherd 1341→1331, bowl 830→821, iron ingot
842→833, emerald 837→828, explode sound 602→596, llama 127→128
(trader), saddle 793→784, shulker box 545→536, decorated pot
301→299, chest 313→311, chain 371→369, music disc cat
1221→1211, pigstep 1237→1227, mall 1227→1217, goat horn
1256→1246, beehive 1276→1266, sentry trim 1323→1313, blast
furnace station 1260→1250, stone slab 277→275, beehive block
865→858, decorated pot block 1083→1076, falling block 49→50,
pig 94→95, dolphin 33→34, mooshroom 82→83, minecart 81→82,
glow item frame 57→58, item frame 70→71, painting 88→89, horse
63→64, and the packet-id decode sites (block_update is 9
pinned, 8 in 26.2; animate is 3 pinned, 2 in 26.2).

## Oracle port: 5 errors, three causes

`horizon-prepare` (commit 6d7959d) ports the oracles to the
1.21.3 APIs — verified with `javap` against the 1.21.3 named
jars, not guessed:

- **`AttributeMap.resetBaseValue` does not exist in 1.21.3**
  (it arrived in 1.21.4 with the `attribute ... base reset`
  command). The attribute and health oracles replicate it from
  the 1.21.4 bytecode: false when the supplier lacks the
  attribute, else the instance's base back to the *supplier's*
  base value (not the attribute default). The health oracle
  keeps the player supplier on its `Data` for the reset case.
- **1.21.3's `Level` has no `dragonParts`**: the landing
  oracle drops that stub override.
- **The cuboid model oracle reads 1.21.3's `BlockModel`**:
  the protected `textureMap` field (`Either<Material, String>`
  per key — a sprite, or a `#name` reference) instead of
  `getTextureSlots()`, the `parentLocation` field instead of
  `getParentLocation()` (which does not exist in 1.21.3), and
  `hasAmbientOcclusion()` instead of `getAmbientOcclusion()`.
  `getElements()` is public in 1.21.3, so it is called
  directly.

The first `horizon-prepare` run surfaced 5 javac errors in 4
files; the second run completed end-to-end.

## Verification

- `cargo check --workspace --all-targets`: clean, 0 errors,
  0 warnings (16 m 53 s at `-j 1`, post-rebase run, with
  `CARGO_INCREMENTAL=0`).
- `cargo test -p horizon-protocol`: **424 pass / 2 fail** (426
  total — the same count as 1.21.4: the rewire replaced the
  `PlayerLoaded` round-trip case with a `PickItem` one). The 2
  failures are the same pre-existing sandbox-network pair in
  untouched code
  (`connection::tests::a_dead_address_does_not_hold_up_the_next`,
  `resolver::dns::tests::asks_each_server_in_turn_and_retries_truncated_replies_over_tcp`),
  out of scope.
- `horizon-prepare` completes end-to-end: exit 0, "Done in
  641.2 s", after the oracle port above.

## Harness subset: no local profile (skipped, CI to measure)

No local harness profile was measured. The 1.21.6 publish
attempted a local sweep and spent ~4 h 30 min on attempts that
produced no profile (the `fast`-profile horizon-client build
OOMs on this VM's 6 GB under 3-stream load); the programme has
since moved to measuring profiles on CI, and disk quota pressure
(~98G of ~100–103G on 2026-11-19) rules out the local sweep's
multi-GB build trees. The profile is to be measured by the CI
`port-verify` workflow on GitHub runners (16 GB) after merge.

`status.json` records the harness section as 0/0/0 of 17 with
this note, and the CI run will fill it in.

## Known gaps (mechanical-layer limits, for the behaviour port)

- **Tooltip hiding is lost writing to 1.21.5+ servers**: the
  marker is dropped by `write_patch`'s `has` filter (no DOWN
  synthesis — the target's `tooltip_display` id differs by
  version and the generated `Ids` struct has no name→id lookup).
  Inherited from 1.21.4.
- **Per-component show flags don't round-trip through saves**:
  `components_tag` keeps emitting the synthesized 1.21.5-form
  `minecraft:tooltip_display` compound, which a 1.21.3 save
  would carry as an unknown component (vanilla drops unknown
  components on load). The 1.21.3 save form (per-component
  `show_in_tooltip: false`, separate hide booleans) is
  behaviour-port work. Inherited from 1.21.4.
- **`skip_dropped` reads the EitherHolder form unconditionally**
  for the removed components, but 26.1+ sends them as bare
  holders — a 26.1+ stack carrying `damage_type`,
  `chicken/variant` or `provides_trim_material` may misparse.
  Same pre-existing pattern as `damage_type`; untested in this
  tree. Inherited from 1.21.4.
- **Smithing-trim DOWN-write old form not implemented**: the
  write side always writes the new form (pattern id); a
  pre-1.21.5 guest's recipe displays would need a reverse
  pattern→template-item lookup plus the trim-pattern context on
  the guest path. Inherited from 1.21.4.
- **Text components stay JSON strings in NBT at this pin**
  (the 4290 unflatten fix is a 1.21.5 fix): the pin codec reads
  and writes NBT compounds on the wire, and `from_tag` converts
  legacy JSON strings — but anything that reads a 1.21.3 save's
  text components as compounds will find strings instead.
  Inherited from 1.21.4.
- **Equip-on-interact is dropped on the pin wire**: the 1.21.3
  equippable layout has no interact field, so a 1.21.5 stack's
  `equip_on_interact=true` does not survive the pin codec (the
  DOWN write to a 1.21.5 target still round-trips it).
  Inherited from 1.21.4.
- **The native pick-item hook has no call site at this pin**
  (see "Packet rewire" above): it exists only so the native
  mod API generator finds its signature. It returns with the
  pick packets in 1.21.4.

## What remains, by cause

### Behaviour work from the diff (not started)

The 313 ranked items are the port's work list; **0 of 313 are
closed**. The work is the resin/eyeblossom removals' behaviour
consequences, the 773 changed classes (656 concern Horizon),
the save-format step (4189 → 4082: 0 schemas, 2 fixes, 4
datafix classes), and the 189 harness reruns — unattempted.
The behaviour code in this tree is still the 26.3 base's.

### Harness-oracle era ports

The per-harness oracles carry their era gaps forward from 1.21.4
unchanged — the same oracle failures as 1.21.4 are expected once
the profile is measured, but the counts are unmeasured for
1.21.3.

### Full harness profile

The CI `port-verify` workflow on main measures the profile on
GitHub runners. The local sweep was skipped (see above), so CI is
the first profile for this version, not a confirmation of a local
one.

### D6 playtest

Deferred by the owner ("leave the play test for later") — recorded
as deferred, never claimed.

## Status

Pin set (protocol 768, world 4082); 6 of 6 tables regenerated
(5 regenerated, vanilla_command_order verified unchanged);
packet rewire done (3 packets removed, 1 restored, 25
renumbered) and the game-side wire boundary reworked (client
pick resolution, loaded-from-join); adapter ladder re-based to
768 (transient DOWN hand-patch, creaking-heart aliases flipped);
component codec re-pinned (custom_model_data int form,
particle codec, `Quirks::PINNED` fixed); oracles ported to the
1.21.3 APIs (4 files); workspace compiles (0 errors, 0
warnings); protocol tests 424 pass / 2 fail (the
sandbox-network pair); prepare verified ok end-to-end.
Harness profile: **not measured locally** (skipped — CI to
measure). D6 deferred (owner).
