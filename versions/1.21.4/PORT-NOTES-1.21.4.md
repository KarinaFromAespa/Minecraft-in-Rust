# Port notes: Minecraft 1.21.4

1.21.4 ("The Garden Awakens") is the seventh adjacent delta port in the
**Components era (E6, 1.20.5–1.21.11)**, taken from 1.21.5. It is a
cross-protocol delta: protocol **770 → 769**, world version 4325 →
4189, data pack 71.0 → 61.0 and resource pack 55.0 → 46.0 — and the
packet set changes both ways: 3 test-block packets are removed,
`minecraft:add_experience_orb` comes back, and 117 play clientbound
packets are renumbered, so this port needed a full packet rewire plus
the adapter re-base. The measured delta is **548 ranked work items**.
The port is the mechanical layer: the pin is set, the six generated
tables are regenerated against 1.21.4's own jar, the adapter ladder is
re-based to the new protocol, the component codec is re-pinned to the
1.21.4 registry, the verification gates pass, and the remaining work
is sized, ranked and documented. The behaviour code in this tree is
still the 26.3 base's, inherited through 1.21.11, 1.21.10, 1.21.9,
1.21.8, 1.21.7, 1.21.6 and 1.21.5; the port is **in progress**, not
parity-verified.

## Provenance

- Base: the programme's completed 1.21.5 tree — the `port/1.21.5`
  branch at fd79ef0, which contains the 1.21.5 tree including the
  1.21.8 code changes (packet rewire, adapter re-base, oracle port),
  the jsr305 jar, the both-forms `gen_version_consts.py`, the
  jsr305 classpath fix in `tools/port_regen.py`, and the full
  ported oracle. No overlay was needed. Published 1.21.5:
  main@7ff1eace.
- 1.21.4's facts, from its own version JSON and generated tables:
  protocol **769**, world (data) version **4189**, data pack
  format **61.0**, resource pack format **46.0**, Java **21**,
  stable release (2024-12-03).
- Prepare ran clean from the Mojang manifest (2026-11-19):
  **8,857 classes indexed, 72,101 methods**; mappings: classes
  **8,857 mapped / 8,857 total (8,829 renamed)**, methods
  **78,666 mapped / 78,666 total (59,634 renamed)**, fields
  **40,134 mapped / 40,134 total (39,880 renamed)**,
  0 placeholders; named jar 33,218,199 bytes; datagen ran.
  Nothing Mojang-derived is committed in this tree.
- Diff 1.21.5 → 1.21.4: **548 ranked work items** (rank 1: 10,
  rank 2: 348, rank 3: 149, rank 4: 39, rank 5: 2). Classes +205
  −496, 25 moved, **1,842 changed (9,139 methods); 1,628 of the
  changed or removed ones are ported or proved by Horizon**.
  Data files +17 −113 ~1,435; assets +80 −281 ~147; en_us keys
  +0 −161 ~44.

## The delta: test blocks, garden content and 31 components go away

Rank 1 is the version pin (`version.rs`), the six regenerate
items, the save-format item (DataFixers 4325 → 4189: 0 schemas,
4 fixes added, 93 datafix classes changed) and the version facts
(protocol 770 → 769, world 4325 → 4189). Rank 2 is 348 harness
reruns. Rank 3 is 149 unproved items. Rank 4 is 39 items. Rank 5
is 2 items.

The visible content changes are the removals of 1.21.5 features:

- **Test blocks**: the `test_block` and `test_instance_block`
  blocks (and their block entities, the `test_environment_definition_type`,
  `test_function` and `test_instance_type` registries), the
  `minecraft:set_test_block` / `minecraft:test_instance_block_action` /
  `minecraft:test_instance_block_status` packets.
- **Garden content**: bush, short/tall dry grass, cactus flower,
  wildflowers, leaf litter and firefly bush (blocks and items),
  the `firefly` and `tinted_leaves` particles, and 52 sound
  events (the registry gains 1).
- **Blue and brown eggs** (items 970/971).
- **31 data components**, among them the whole variant family
  (chicken/cow/pig/wolf/fox/frog/horse/llama/cat/parrot/rabbit/
  salmon/tropical-fish/mooshroom/axolotl/villager variants,
  wolf sound variant, wolf/sheep/shulker/cat collars and colours),
  `tooltip_display`, `weapon`, `blocks_attacks`,
  `potion_duration_scale`, `break_sound`, `provides_trim_material`
  and `provides_banner_patterns`. The pin gains
  `hide_additional_tooltip` (15) and `hide_tooltip` (16) in their
  place. The component registry is 67 entries (0–66).
- **Potion entities**: `splash_potion` (99) and `lingering_potion`
  (100) are gone; the single `potion` entity (99) is back.
- **The creaking heart** goes back to a boolean `active` (12
  states, active-major) from 1.21.5's `creaking_heart_state`
  (18 states).

The item id shift rule across the delta: ids <204 unchanged;
205–207 shift −1; 211–258 shift −4; 261–328 shift −6; 330–831
shift −7; 834–969 shift −9; 972+ shift −11. Entity ids above 100
shift −1 (99 is potion in both). Block ids: <133 unchanged;
136–266 shift −3; 268–865 shift −4; 868–1049 shift −6; 1052–1102
shift −8; >1103 shift −9. Attributes are unchanged. That is why
the adapter re-base and the test re-pins below are mechanical
but wide.

## Tables: 6 regenerated

| Generator | Result vs 1.21.5 |
|---|---|
| `version_consts` | regenerated (+8 −8 — the pin: 1.21.4 / 769 / 4189 / 61.0 / 46.0) |
| `packet_ids` | regenerated (+125 −129 — 3 packets removed, 1 added, 117 renumbered) |
| `entity_tracking` | regenerated (+1 −2) |
| `lan_command_order` | regenerated (+7 −86) |
| `command_suggestions` | regenerated (+0 −1) |
| `vanilla_command_order` | regenerated (+4 −1) |

The regen took three runs (commits c70f563, 306672cde4,
b658fe174c on `port/1.21.4`), because each run unblocked the
next: run 1 regenerated the first four tables and failed the two
oracle-command generators — `command_suggestions` on NBT API
errors in the oracles (1.21.4 tags have no `value()`,
`CompoundTag` has no `keySet()`), fixed in 6c88fc6, and
`vanilla_command_order` because `horizon-protocol` would not
compile until the packet rewire landed. Run 2 regenerated
`vanilla_command_order` and failed `command_suggestions` on
`TagParser.parseCompoundFully` (a later addition; 1.21.4 has
only `parseTag`) — fixed in b12fa91 (17 sites in 13 files).
Run 3 regenerated `command_suggestions`. The committed
`regen.json` lists all six generators with their run-1–3 statuses
combined (D1 completeness, per the gate change at main@15ff93a8).
A fresh regen reproduces all six committed tables byte-for-byte —
see `regen-determinism.json`, no overrides.

## Packet rewire: 3 packets removed, 1 restored, 117 renumbered

Commit 65307b5 on `port/1.21.4` removes the 3 test-block packets
(`SetTestBlock`, `TestInstanceBlockAction`,
`TestInstanceBlockStatus`, with `TestInstanceData`) and adds
`AddExperienceOrb` (varint id, 3 doubles, short value — verified
with `javap -c` against the 1.21.4 named jar). Two test-block
tests went with the removed packets; the `test_action` helper in
`step_1_21_6.rs` and the now-empty `translate::v1_21_6` module
went too. An `AddExperienceOrb` round-trip test was added later
(commit b191fd8).

Commit 56e787f reworks the game-side wire boundary around the
missing packets: `remote.rs` drops the `TestInstanceBlockStatus`
arm and gains an `AddExperienceOrb` arm (spawns the orb, then
overrides its value — `spawn_entity` already handles
`minecraft:experience_orb` with value 1); `lan_actions.rs`
drops the test-block arms; `test_blocks.rs` and
`test_instances.rs` drop the LAN send/apply functions; the
test-block editors (`test_block_edit.rs`,
`test_instance_edit.rs`) now apply their values locally and
return a new `Outcome::Close` ("close without sending") instead
of sending packets — `Game::send` is a no-op without a remote
connection, so the packets were LAN-only and local-only edits
are honest. The test-runner operations only the removed apply
function invoked (`save_test`, `export_test`, `run_test`,
`TestDef.description`, `track_and_start_runner`) are removed as
dead code: the packets arrived in 1.21.5 and stay absent down
the chain.

## Adapter re-base to 769

The adapter ladder (`crates/horizon-protocol/src/adapter/`) is
re-based from the 770 pin to 769 (commit 2d3e575 on
`port/1.21.4`; scripts `rebase769.py` and `fix_slot_display769.py`
in the Stream 1 workdir, modelled on the 1.21.5 `rebase770.py`).
The rebase was recovered from diffing the regenerated tables
against the 770 tree:

- INHERIT map: 1.21.3→PIN, 1.21.5→PIN, 1.21.6→PIN, 1.21.8→PIN.
- Potion renames: `splash_potion→potion` added for 1.21.5,
  1.21.6, 1.21.8, 1.21.10, 1.21.11, 26.1.2, 26.2
  (`lingering_potion` has no pin equivalent and drops to NONE);
  the old `potion→splash_potion` entries for 1.20.2/1.20.3/
  1.21.1/1.21.3 removed (the pin now has `potion`).
- Creaking-heart state aliases rewritten for 1.21.4's boolean
  `active` (12 states, active-major): both quiet states
  (dormant, disabled) map to `active=false, natural=false`,
  with dormant first so the generator's DOWN prefers it; the
  v1_21_3 BLOCK_STATE_DOWN patch inserts length-1 runs for the
  natural=true halves (they cannot merge into length-2 runs —
  two pin states share one old target).
- **Entity-data serializer remaps recomputed** (unlike 770/771):
  1.21.5 added 4 variant serializers (cow, wolf sound, pig,
  chicken) at 1.21.5-order ids 23, 25, 27, 28; the pin has 31
  serializers (0–30). V1_21_4's custom remap is removed
  (generated Same); V1_21_8/6/5, V1_21_10/11, V26_2 and V1_20_3
  gained the recomputed remaps; V1_21_3/1/20_6 dropped theirs
  (their order matches the pin).
- `adapter.rs` pin swap: V1_21_4 is now the pin identity static
  (quirks merged: `wolf_variant_holder`, `components_before_1_21_5`,
  `components_before_1_21_6`); V1_21_5 is a regular old version;
  the `from_protocol` test updated (774 five above pin, 773 four,
  772 three, 771 two, 770→V1_21_5, 769→Current).
- SlotDisplay: the `slot_display` registry is identical in 1.21.4
  and 1.21.5 (8 entries, same order), so the 770 fix values apply
  unchanged; the v1_20_3 ENTITY_TYPE_DOWN anchor (breeze_wind_charge
  still 18 in the 769 pin) is identical too.

## Component re-pin: the 1.21.4 registry

The `item::component` constants were still the 1.21.5 pin's ids
(commit 47d621a on `port/1.21.4`). All 67 entries of the 1.21.4
`data_component_type` registry are now pinned (0 custom_data …
66 container_loot), the 31 removed components are gone from the
constants (BLOCKS_ATTACKS, PROVIDES_TRIM_MATERIAL,
PROVIDES_BANNER_PATTERNS, CHICKEN_VARIANT deleted outright),
and `TOOLTIP_DISPLAY` is now an internal marker (−1003): 1.21.4
has no `tooltip_display` component.

**Tooltip design.** The pin expresses tooltip hiding as two Unit
components (`hide_tooltip`, `hide_additional_tooltip`) plus a
`show_in_tooltip` bool at the end of each TooltipProvider
component's codec (verified with `javap`: Unbreakable,
ItemEnchantments ×2, AdventureModePredicate ×2,
ItemAttributeModifiers, DyedItemColor, ArmorTrim, JukeboxPlayable
— the last was missing from the old-server flag list, a latent
bug fixed here). `Component::TooltipDisplay` stays the canonical
in-memory form (the client reads it via `stack.component` and the
NBT form `minecraft:tooltip_display`); the pin codec synthesizes
it at the stack level in `read_stack`/`read_template`/
`WirePatch::decode`, and `write_patch`/`DelimitedSlot::write`
split it back into the two hide booleans plus per-component show
flags. Writing to a 1.21.5+ server drops the hiding — the marker
has no wire id there (and the target's `tooltip_display` id is
15 in 1.21.5–1.21.10 but 18 in 1.21.11+, so it cannot be
hardcoded). See "Known gaps" below.

**`Quirks::PINNED` fixed.** It still said "the pinned 1.21.8's
own wire layouts" with `components_before_1_21_6: false` — stale
since 1.21.8. For the 1.21.4 pin it is now
`components_before_1_21_6: true, components_before_1_21_5: true,
wolf_variant_holder: true`, matching V1_21_4.quirks. (The
published 1.21.5 tree carries the same staleness — flagged for a
fix-up there; the 1.21.6/1.21.7 pins are unaffected, both
post-date 1.21.6.)

**Real code fixes surfaced during the re-pins** (commits
17f3a77, 47d621a, 557782e, b385cce, 9989409, 51307c8, d053132,
f51456c, af3cf6f, 5562ad8, b4a2f78, 14be1b1, d5c1710, db4813d,
b191fd8, ca20060):

- `ARGUMENT_TYPES` in `packets/play/commands.rs`: 55 → 54
  entries (`resource_selector` 47 removed).
- Particle codec re-pinned (real code, `entity_data.rs`): trail
  47, vibration 46, item 45, item_slime 48, shriek 101, sculk
  35/36, instant_effect 44 — and the Item arm now covers both
  `minecraft:item` and `minecraft:item_slime` (the 1.21.5 pin
  only covered item; item_slime fell through to None).
- Entity-data serializer constants re-pinned from the 1.21.4 jar
  (`javap -c` on `EntityDataSerializers.<clinit>`: 31 calls,
  SNIFFER before ARMADILLO); the four 1.21.5 variant serializers
  removed.
- `v1_21_5::write` now gates on the *target's* layout
  (`quirks_written().components_before_1_21_6`), so pin writes
  drop the 1.21.6 display/shearing fields the pin reader cannot
  parse; the `before_5` check is direction-aware (pin reads of
  equippable no longer consume a phantom interact byte).
- `DelimitedSlot::write` mirrors `write_patch`'s tooltip
  synthesis (it previously wrote the −1003 marker raw, which the
  delimited read rejected).
- Smithing-trim old-form read now requires an active UP session
  with `components_before_1_21_5`: the pin codec reads the old
  form only when the translator holds the trim-pattern registry
  (the game decodes pinned bytes with no such context).
- Legacy stacks: `from_tag` parses JSON-string text components
  to NBT (the 4290 unflatten fix does not run at data version
  4189 — see below), synthesizes the tooltip marker from
  `show_in_tooltip: false` fields and HideFlags, and reads
  `dyed_color` from its 1.20.5 compound form; `json_text` no
  longer double-encodes already-JSON strings; `legacy_tag` and
  `legacy_raw` re-pinned.
- The v1_20_3 block-entity oracles were wrong for this pin, not
  the code: the text-component unflatten fix is keyed 4290 (a
  1.21.5 fix), so at 4189 `CustomName` and sign messages stay
  JSON strings in NBT — the oracles' "after" fixtures were
  rewritten to the JSON-string form (commit af3cf6f).
- The shearing-sound placeholder is now `entity.sheep.shear`
  + 1 (1256): 1.21.4 has no `item.shears.snip` (1328 there is
  `block.resin.break`).
- Hash tables (`item/hash/raw.rs`, `gameplay.rs`) re-pinned to
  the 1.21.4 component ids; 23 removed-component cases dropped.
- 71 pinned component fixtures regenerated from the pin codec
  (TEMP-REGEN method); the legacy-slots fixtures likewise (39
  labels, 58 replacements).

The registry shifts forced wide test re-pins: diamond 845→836,
brick 963→954, calcite state 23344→23329, filled map 1042→1031,
tadpole 124→123, turtle 131→130, zombie villager 146→145, player
kind 148→147, angler sherd 1352→1341, bowl 839→830, iron ingot
851→842, emerald 846→837, explode sound 606→602, llama
128→127 (trader), saddle 800→793, shulker box 552→545,
decorated pot 307→301, chest 319→313, chain 378→371, music disc
cat 1232→1221, pigstep 1248→1237, mall 1238→1227, goat horn
1267→1256, beehive 1287→1276, sentry trim 1334→1323, blast
furnace station 1271→1260, beehive block 871→865, decorated pot
block 1091→1083, and the packet-id decode sites (block_update
is 9 pinned, 8 in 26.2; animate is 3 pinned, 2 in 26.2).

## Oracle port: 52 errors, then 2, then the datafix exporter

`horizon-prepare` (commits 76ce485, 556297a, 7d8ef28, 058ad0a)
ports the oracles to the 1.21.4 APIs — verified with `javap`
against the 1.21.4 named jar, not guessed:

- `Player.isCreative()` and `isSpectator()` are **abstract** in
  1.21.4, and `Player` has **no `gameMode()` method** (nor does
  `ServerPlayer` — it has a `gameMode` field). All 40
  `gameMode()` overrides were removed: classes that already had
  isCreative/isSpectator kept them; the rest gained `false`
  pairs; the fixture-driven ones derive from `mode`
  (BlockEntityAnim, HeldDrop, SpawnRules, NaturalSpawn, Raid);
  DamageSource's Human returns `getAbilities().instabuild`.
- `Entity.causeFallDamage` and `Block.fallOn` take a **float**
  distance in 1.21.4 (8 overrides + the fallOn call and its
  reflective lookup in VanillaBlockLandingOracle).
- `Level.getFreeMapId()` and `setMapData()` are abstract;
  `playSeededSound` takes a **Player** first parameter;
  `LevelAccessor.levelEvent` is `levelEvent(Player, int,
  BlockPos, int)`.
- `CompoundTag` has `getString` but no `getStringOr`;
  `VoxelShape.move` takes a Vec3 (or three doubles), not a
  BlockPos.
- **Model oracles** (VanillaCuboidModelOracle,
  VanillaBlockStateBindingOracle): 1.21.4 predates the 1.21.5
  model refactor — no `BlockStateModel`, `MultiPartModel`,
  `BlockStateDefinitions`, `SimpleUnbakedGeometry`, or
  `BlockModelDefinition.CODEC`. Ported to the pre-refactor API:
  `BlockElement`/`ItemTransform` public fields, `BlockFaceUV`
  (rotation in degrees on the uv object), `BlockModel`
  getters (`getElements`/`getParentLocation` via reflection —
  package-private), `BlockModelDefinition.fromJsonElement` +
  `instantiate(StateDefinition, String)`, `MultiVariant`/
  `MultiPart` roots with `visualEqualityGroup` (an IntList of
  matching selector indices, confirmed by bytecode), and the
  state-definition mapper straight from the block registry
  (`Registry.getValue`, which returns the value directly —
  `get` returns an Optional in 1.21.4).
- **Datafix exporter**: 1.21.4's `BlockStateData.register` takes
  SNBT strings, `(int, String, String...)`, not Dynamics — the
  hook descriptor, the `record` signature and the replay check
  (value equality, null-safe) were ported accordingly; the
  recorded strings are re-parsed with vanilla's own `parse`.

The first `horizon-prepare` run surfaced 52 javac errors in 6
files; the second run surfaced 2 more; the datafix exporter
then failed at runtime (twice — the SNBT register, then a
null replay entry). The fix batches above cover the latent
occurrences of the same API shapes across the whole oracle tree
(the exporters compile in batches, so each run's errors were
only the first batch).

## Verification

- `cargo check --workspace --all-targets`: clean, 0 errors,
  0 warnings (11 m 56 s at `-j 1`, post-rebase run, with
  `CARGO_INCREMENTAL=0`; the only warning in the run was a dead
  `float` helper in `item/hash/raw.rs`, removed in ca20060).
- `cargo test -p horizon-protocol`: **424 pass / 2 fail** (426
  total — down 3 from 1.21.5's 429: the packet rewire removed 4
  test-block tests and added 1 `AddExperienceOrb` test). The 2
  failures are the same pre-existing sandbox-network pair in
  untouched code
  (`connection::tests::a_dead_address_does_not_hold_up_the_next`,
  `resolver::dns::tests::asks_each_server_in_turn_and_retries_truncated_replies_over_tcp`),
  out of scope.
- `horizon-prepare` completes end-to-end: exit 0, "Done in
  167.0 s", after the oracle port above.

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

## Known gaps (mechanical-layer limits, for the behaviour port)

- **Tooltip hiding is lost writing to 1.21.5+ servers**: the
  marker is dropped by `write_patch`'s `has` filter (no DOWN
  synthesis — the target's `tooltip_display` id differs by
  version and the generated `Ids` struct has no name→id lookup).
- **Per-component show flags don't round-trip through saves**:
  `components_tag` keeps emitting the synthesized 1.21.5-form
  `minecraft:tooltip_display` compound, which a 1.21.4 save
  would carry as an unknown component (vanilla drops unknown
  components on load). The 1.21.4 save form (per-component
  `show_in_tooltip: false`, separate hide booleans) is
  behaviour-port work.
- **`skip_dropped` reads the EitherHolder form unconditionally**
  for the removed components, but 26.1+ sends them as bare
  holders — a 26.1+ stack carrying `damage_type`,
  `chicken/variant` or `provides_trim_material` may misparse.
  Same pre-existing pattern as `damage_type`; untested in this
  tree.
- **Smithing-trim DOWN-write old form not implemented**: the
  write side always writes the new form (pattern id); a
  pre-1.21.5 guest's recipe displays would need a reverse
  pattern→template-item lookup plus the trim-pattern context on
  the guest path.
- **Text components stay JSON strings in NBT at this pin**
  (the 4290 unflatten fix is a 1.21.5 fix): the pin codec reads
  and writes NBT compounds on the wire, and `from_tag` converts
  legacy JSON strings — but anything that reads a 1.21.4 save's
  text components as compounds will find strings instead.
- **Equip-on-interact is dropped on the pin wire**: the 1.21.4
  equippable layout has no interact field, so a 1.21.5 stack's
  `equip_on_interact=true` does not survive the pin codec (the
  DOWN write to a 1.21.5 target still round-trips it).

## What remains, by cause

### Behaviour work from the diff (not started)

The 548 ranked items are the port's work list; **0 of 548 are
closed**. The work is the test-block/garden/egg/component
removals' behaviour consequences, the 1,842 changed classes
(1,628 concern Horizon), the save-format step (4325 → 4189:
0 schemas, 4 fixes, 93 datafix classes), and the 348 harness
reruns — unattempted. The behaviour code in this tree is still
the 26.3 base's.

### Harness-oracle era ports

The per-harness oracles carry their era gaps forward from 1.21.5
unchanged — the same oracle failures as 1.21.5 are expected once
the profile is measured, but the counts are unmeasured for
1.21.4.

### Full harness profile

The CI `port-verify` workflow on main measures the profile on
GitHub runners. The local sweep was skipped (see above), so CI is
the first profile for this version, not a confirmation of a local
one.

### D6 playtest

Deferred by the owner ("leave the play test for later") — recorded
as deferred, never claimed.

## Status

Pin set (protocol 769, world 4189); 6 of 6 tables regenerated;
packet rewire done (3 packets removed, 1 restored, 117
renumbered) and the game-side wire boundary reworked; adapter
ladder re-based to 769; component codec re-pinned to the 1.21.4
registry (tooltip synthesis, `Quirks::PINNED` fixed); oracles
ported to the 1.21.4 APIs (44 files + 3 follow-ups); workspace
compiles (0 errors, 0 warnings); protocol tests 424 pass / 2 fail
(the sandbox-network pair); prepare verified ok end-to-end.
Harness profile: **not measured locally** (skipped — CI to
measure). D6 deferred (owner).
