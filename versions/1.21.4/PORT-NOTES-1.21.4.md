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

End main island (worldgen-verify): vanilla 1.21.4's
`minecraft:end_islands` includes the central island cone in the
function itself, but the tree carried the 26.3 outer-only sampler
and did not decode the `end_islands` name at all, so the End
pipeline failed to build. Carried 26.2's fix:
`GraphNode::EndIslands { central }`, with `end_islands`
decoding to central: true and `end_outer_islands` to central:
false; the cone formula was verified against the 1.21.11 named
jar's EndIslandDensityFunction bytecode. The end-to-end worldgen
test (`prepared_world_builds_and_generates_chunks`, run by the
worldgen-verify CI job) now generates two chunks per dimension
and dumps them as JSON; it caught this gap.

## Behaviour port — rank 1 (10 items, CLOSED 2026-11-24)

- Item 1 components (PORTED): the wire layer was already 1.21.4's
  (`crates/horizon-protocol/src/item/v1_21_4.rs` codec,
  `HIDE_ADDITIONAL_TOOLTIP = 15`, `HIDE_TOOLTIP = 16`, synthesized
  `TOOLTIP_DISPLAY = -1003`), but `DATA_COMPONENT_TYPES` in
  `crates/horizon-assets/src/item_model.rs` was the stale 26.3
  122-entry list, still containing all 31 removed components (the
  1.21.5 tree's list is stale the same way — the 1.21.5 port never
  rebuilt it). Rebuilt to the 67-entry 1.21.4 list, alphabetically
  sorted, jar-verified from `DataComponents.<clinit>` in the 1.21.4
  named client jar: 67 ids in registration order, [15] =
  hide_additional_tooltip, [16] = hide_tooltip, none of the 31
  removed names present, both added names present. Behaviour
  fallout — code that reads the removed components by name —
  belongs to the rank-2 items for those modules.
- Item 2 protocol (verified): all 131 play clientbound + 62 play
  serverbound ids in `crates/horizon-protocol/src/ids.rs` match the
  report's renumbering table exactly (0 errors across the 117
  renumbered, 3 added, 3 removed); zero test-block packet remnants
  in the file.
- Items 3–8 regenerate (verified): `regen-determinism.json` shows all
  six generators reproduce the committed tables byte-identical, with
  no overrides.
- Item 9 save-format (PORTED from the 1.21.11 template's final state,
  with 1.21.4 adjustments — the full removal, not the 1.21.10 port's
  always-false stub): 1.21.4 has no `FileFixerUpper`
  (`net/minecraft/util/filefix` is absent from both named jars) and
  no `FILE_FIXING_REQUIRED` backup status (`LevelSummary$BackupStatus`
  is only NONE/DOWNGRADE/UPGRADE_TO_SNAPSHOT, javap-verified from the
  named client jar). Deleted fix_1493/4772/4773/4899,
  `level_dat_to_saved_data` and their helpers
  (`resolve`/`move_file`/`delete`/`dimensions`, `read_level`/
  `write_level`/`write_saved_data`/`read_saved_data`),
  `requires_file_fixing` and the `FILE_FIXES` constants; the
  whole-world upgrade follows the pre-26.x layout (overworld at the
  root, DIM-1/DIM1, `playerdata/`, `advancements/`, `stats/`);
  `current_world_gen_settings` reads `WorldGenSettings` from
  level.dat's `Data` only; `saved_data_type` keeps only the types
  1.21.4's `DataFixTypes` defines — the 1.21.11 set minus
  stopwatches (no `Stopwatch` class in the jar) and minus
  world_border (no world-border saved data in 1.21.4; the border
  lives in level.dat) — with `ForcedChunksSavedData.FILE_ID =
  "chunks"` javap-verified for the flat `chunks.dat` mapping.
  `NbtUtils.getDataVersion` in 1.21.4 has only the two-argument
  `(CompoundTag, int)` form (no `(Dynamic, int)` form yet).
  `WorldUpgrader` has the inner classes (`AbstractUpgrader`,
  `ChunkUpgrader`, `SimpleRegionStorageUpgrader`, `EntityUpgrader`,
  `PoiUpgrader`, `FileToUpgrade`, `DimensionToUpgrade`),
  javap-verified, so the 1.21.11 doc shape applies. The 4 "added"
  fixes are already implemented in the Rust table in their 1.21.4
  (split) forms: `BlockEntitySignTextStrictJsonFix` +
  `ItemWrittenBookPagesStrictJsonFix` at key(165, 0) (1.21.5
  consolidated them into `TextComponentStrictJsonFix`),
  `ObjectiveDisplayNameFix` + `TeamDisplayNameFix` at key(1514, 0)
  (1.21.5 consolidated them into `ScoreboardDisplayNameFix`).
  `DataFixers`: 1.21.4's fix versions (177 ≤ 4189) are a subset of
  1.21.5's (191 ≤ 4325) except key 2537 (`legacy_dimension_id`,
  kept per the 1.21.11/1.21.10 templates — the 1.21.5 tree lacks
  it); the 15 1.21.5-only versions are all > 4189 (4293–4314, the
  1.21.5-release fixes), so the CURRENT-gated fix table is correct
  as-is. The 93 changed datafix classes are mechanical noise only
  (sampled `FixWolfHealth` — a package move into `fixes/`;
  `BlockEntityIdFix` — `ID_MAP` public→private;
  `ItemStackTagFix` — the Typed→Dynamic API reshape plus lambda
  renumbering — no behaviour change), so the Rust fix
  implementations are untouched. Files:
  `datafix/{files,mod,progress,world_upgrader,tests}.rs`,
  `datafix/fixes_legacy.rs` (doc comments naming the 1.21.4 fix
  classes), `save.rs` (`needs_file_fixes` deleted,
  `world_gen_settings` reads level.dat only, test replaced with the
  level.dat-through-the-fix-pipeline form), `title.rs`
  (`WorldEntry.file_fixing` field deleted, `can_edit` /
  `should_backup` simplified, the file-fixing backup branch and
  the `selectWorld.upgrade_and_play` button gone, the backup
  prompts' file-fixing arms replaced with `unreachable!`, the
  file-fixing tooltip block deleted, `select_tooltip` loses its
  `selected` parameter), `title/tests.rs` (the file-fixing
  optimize test deleted). NOT in item 9: the `save.rs` settings
  store still writes `world_gen_settings.dat` / `world_clocks.dat`
  / `game_rules.dat` as saved-data files (26.x) instead of
  level.dat's `Data` — that rewire is rank-2 save_roundtrip work,
  as in 1.21.11.
- Item 10 facts (verified): `version.rs` reads 1.21.4, protocol 769,
  data version 4189, data pack 61.0, resource pack 46.0, Java 21,
  stable, build time Tue Dec 03 10:09:48 UTC 2024.

## Behaviour port — rank 2

Triage (2026-11-24): a batched javap jardiff of the 1.21.5 and
1.21.4 named client jars (1,565 rank-2/3 classes: 1,333 common, 7
added, 225 removed, 1,313 method-changed, 0 javap failures) crossed
with a per-module diff of the 1.21.5 and 1.21.4 trees (248 identical,
145 differs, 102 widely-named, 2 only-in-1.21.4) gives 42 no-change,
13 carry-over and 293 review. The semantic diff over the 962 changed
named methods of the review items classifies 719 real, 101 same, 131
missing-new, 11 missing-old; `real_diff.py` clusters the real ones by
shape for the review batches.

- Batch 1 (42 no-change): items 90, 172, 183, 190, 194, 233, 235,
  236, 243, 244, 247, 250, 257, 262, 263, 271, 273, 275, 282, 283,
  284, 285, 290, 291, 295, 296, 297, 298, 299, 300, 317, 318, 320,
  321, 322, 324, 325, 327, 328, 330, 331, 358. Each module is
  identical between the 1.21.5 and 1.21.4 trees (0 diff lines) and
  no named method changed 1.21.5→1.21.4 per jardiff (verified
  programmatically across all 42: 0 named-method hits, including
  class removals). Rank 2: 42/348.

- Batch 2 (8 of 13 carry-over): items 189, 216, 227, 229, 242,
  272, 294, 329. Each module's 1.21.5 form carried with jar
  evidence; no named method changed 1.21.5→1.21.4 for any of them.
  189 item_model_fixture.rs: the `oversized_in_gui` fixture field
  removed (1.21.6-only; `oversized` occurs only in RegionFile and
  RealmsDownloadLatestWorldScreen in both jars). 216
  remote_mob_data.rs: the HappyGhast STAYS_STILL/IS_LEASH_HOLDER
  arms removed (0 HappyGhast classes in the 1.21.4 jar); test
  renamed, happy ghast → cow. 227 perspective.rs: comment-only,
  the `camera_distance` comments reworded to 1.21.4 (the attribute
  is 1.21.6-only; the 1.21.4 distance is a flat 4× scale). 229
  game/recipe_book.rs: `Call::Stratum` removed (nextStratum added
  in 1.21.6; Stratum absent from both jars), with its fixture arm.
  242 collision_context.rs: `placement()`/`with_position()`
  removed (1.21.4 `CollisionContext` has only `empty`/`of`/`of`;
  `placementContext` arrived in 1.21.5, `withPosition` in 1.21.6),
  fixture/oracle/drivers trimmed; the `placement` flag stays,
  always false. Caveat: the oracle still calls
  `emptyWithFluidCollisions()` and `positionContext(y)`, absent
  from both jars — pre-existing in the 1.21.5 oracle too, and the
  collision harnesses are not in the 1.21.5 pin profile, so oracle
  compile status is unproven until a CI harness run. 272
  functions.rs: `create_pack`/`is_portable_name` and the
  `/datapack create` node removed (`create` is not a
  DataPackCommand literal in either jar). 294 lan.rs: the dialog
  sync, the waypoint fields and `lan_update_waypoints` call, the
  HappyGhast metadata arms and the dialog test fixtures removed;
  the frog-variant hunk KEPT (pig/cow/chicken variants are
  1.21.5-only), so lan.rs is otherwise byte-identical to 1.21.5.
  329 recipe_book.rs: the 1.21.5 read/write form carried,
  jar-verified (a missing list reads as empty, settings still
  load; all eight open/filter flags written unconditionally), with
  a new `a_book_without_lists_still_loads_its_settings` test.
  The waypoint/locator-bar system removal anchored at item 294
  spans game.rs (the contextual HUD bar → the direct jump/XP bar,
  the Game fields, the mod decls), client_lib.rs, game/packs.rs,
  game/resource_loader.rs, game/remote.rs, game/world_session.rs,
  game/lan_attributes.rs, game/mounts.rs (`mount_jumping`),
  game/commands.rs + commands/extra.rs + commands/admin.rs (the
  `/waypoint`, `/dialog` and `/version` commands),
  attribute_updates.rs, motion.rs (`MotionEvent::Waypoint`),
  item_tooltip.rs (the waypoint attributes, the invisibility
  mapping), gamerules.rs + save.rs (the `locator_bar` gamerule),
  lan/mod.rs, scene_motion_probe.rs, and the deleted files
  game/locator_bar.rs, game/lan_waypoints.rs, lan/waypoints.rs,
  locator_bar.rs. The dialog-system removal (dialog.rs,
  dialog_screen.rs, game/dialogs.rs, screens.rs, remote_chat.rs,
  packets/play.rs `TrackedWaypoint`/`DialogHolder`) is still in
  progress and is recorded under items 274/323. Rank 2: 50/348.

- Batch 2 continued (item 177): `crates/horizon-protocol/src/
  item/hash/raw.rs` verified against the jar-verified 67-entry
  1.21.4 component registry: the mechanical publish had already
  re-pinned every component arm to the 1.21.4 ids and dropped the
  1.21.5-only components (`potion_duration_scale`,
  `provides_trim_material`, `weapon`, `blocks_attacks`,
  `break_sound` — all absent from the 1.21.4 registry) from the
  arms and the test cases. The variant serializer arms (72–95,
  Horizon synthetic ids) are unchanged from 1.21.5; arms 74/84/
  85/86 reference registries 1.21.4 lacks (wolf_sound_variant,
  pig/cow/chicken variants — confirmed via `Registries` javap)
  and return None, the documented no-codec-form outcome; they
  stay because the default arm falls through to
  `gameplay::code` and removing them would change that path.
  The 13 mob-variant test cases the publish's regen dropped
  were restored with their 1.21.5 hashes (the arms are live code
  in 1.21.4, the CRC32C hashes version-independent;
  1.21.5/1.21.10/1.21.11 all keep them). Rank 2: 51/348.

- Batch 2 continued (items 274, 323): the dialog-system removal
  is closed. Item 274 (`packets/play.rs`): the `DialogHolder`
  enum + codec and the waypoint types (`waypoint_operation`,
  `WaypointId`, `WaypointTarget`, `TrackedWaypoint` + codec)
  removed; the `dialog_holder_encodes_registry_as_id_plus_one`
  test removed; the "1.21.5 has no dialog packets" comments
  fixed to "1.21.4" in play.rs, configuration.rs and adapter.rs.
  KEPT, jar-verified: the `AddExperienceOrb` packet + enum entry
  (1.21.4 HAS `ClientboundAddExperienceOrbPacket`; 1.21.5 does
  not — removed in 1.21.5, orbs spawn through AddEntity there)
  and the ABSENCE of the test-block packets (0 test-block classes
  in the 1.21.4 jar). Item 323 (`game/remote_chat.rs`): the
  1.21.5 form carried wholesale — the command-confirmation system
  removed (`CommandCheck` + `verify_command`, `CommandAccept`,
  `Behind`, `CommandPrompt`, `send_unattended_command`) and
  `send_unsigned_command` restored; jar-verified, 1.21.4's
  `ClientPacketListener` has `sendUnsignedCommand`, not
  `sendUnattendedCommand`. The removal spans game.rs (the
  `dialogs` and `command_prompt` fields, the `DialogWarning`
  screen, every `dialog_open` guard), remote_chat.rs,
  net_harness.rs, native_screen_observer.rs, chat.rs, books.rs
  (click-handler hunks), sign_edit.rs, screens.rs,
  server_links.rs, agent.rs, control.rs, core_tests.rs,
  debug_options.rs, play_keys.rs, key_codes.rs, play_input.rs,
  ime.rs, inventory_input.rs, pad.rs, pad_menu.rs,
  pointer_capture.rs, screen_backgrounds.rs, remote.rs
  (`ContainerClose`), narrator.rs (`edit_box_focused` hunk),
  net.rs (`send_custom_click`), widgets.rs (`CHECKBOX_BOX`),
  horizon-client-core text.rs + component.rs,
  horizon-client-render signs.rs + hud_layout.rs,
  horizon-command codec.rs + component.rs + tree_extra.rs (the
  `/dialog` and `/waypoint` commands), lan/data.rs (the
  `minecraft:dialog` registry + `pack_entries` + dialog tests),
  and the deleted files dialog.rs, dialog_screen.rs,
  game/dialogs.rs, game/quick_actions.rs. Final sweep: no
  `dialog_open`/`dialog_key`/`command_prompt`/`DialogHolder`/
  `TrackedWaypoint`/waypoint code refs remain (only the
  component.rs test proving 1.21.6 click actions are dropped,
  pathfinding waypoints, and comments). Rank 2: 53/348.

- Batch 2 continued (items 249, 326): the HappyGhast system
  removal is closed, and batch 2 is complete (13/13 carry-over).
  Item 249 (`mobs/animal_brain.rs`): the 1.21.5 form carried
  wholesale, jar-verified (HappyGhast 0 classes in the 1.21.4
  jar; `FollowTemptation` has only the 2 ctors in 1.21.4, no
  3-arg look-in-eyes form) — the Ghastling brain and the 1.21.6
  struct shapes (`Tracker.block_eye`, `AnimalPanic.fly`,
  `FollowTemptation.look_in_the_eyes`, `FollowAdult.player`,
  `B::reset`) removed. The removal spans mobs.rs, species.rs,
  riders.rs, entity_culling.rs, ai.rs, living.rs,
  living_tests.rs, anim.rs, render.rs, equine.rs, finalize.rs,
  drops.rs, leash.rs, mounts.rs, passengers.rs,
  remote_mob_use.rs, remote_vehicles.rs, remote.rs,
  creatures.rs, mob_sounds.rs + sound_engine.rs, anticheat.rs,
  dispenser.rs, entity_nbt.rs, scoreboard.rs + stats_screen.rs,
  boat_leads.rs, cem.rs, sound_tests.rs,
  block_update_fixture.rs, nether_fossil.rs (`placeDriedGhast`),
  animate_tick.rs, game.rs, and the deleted files
  mobs/happy_ghast.rs + game/ghasts.rs. KEPT, matching 1.21.5:
  the dried-ghast BLOCK handling (animate_tick, horizon-world
  level classes/fluids/place/shapes/tick/timeline) — data-driven
  dead code, the block is not in the 1.21.4 tables. Final sweep:
  0 HappyGhast/happy_ghast/ghastling refs in Rust code. Item 326
  (`mobs/parrot.rs`): the 1.21.5 form carried wholesale,
  jar-verified — 1.21.4's `Parrot.MOB_SOUND_MAP` has exactly 37
  entries, the ender dragon IS imitated (None key) and happy
  ghast/camel husk/parched/zombie horse/zombie nautilus are NOT
  in it. Rank 2: 55/348.

- Batch 3 (items 11 + 155): the entity save form ported to
  1.21.4's lists, jar-verified — the first version in the
  programme BELOW 1.21.5, so the ArmorItems/HandItems list form
  has no template and was written from the 1.21.4 jar.
  `Mob.addAdditionalSaveData` writes exactly 12 keys: the four
  lists (always, empty slots as empty compounds),
  body_armor_item + body_armor_drop_chance (only with a body
  item), DeathLootTable/Seed, CanPickUpLoot, LeftHanded, NoAI,
  PersistenceRequired — no `drop_chances` compound (no
  `DropChances` class), no leash/home keys. Equines and camels
  save `SaddleItem`, pigs and striders a `Saddle` boolean, and
  there is no SADDLE slot. Armor stands write the same lists.
  `FallDistance` is a FLOAT in 1.21.4 (`putFloat`; 1.21.5
  renamed it to a double `fall_distance`) — renamed across
  entity_nbt.rs, save.rs (player), persist.rs, commands/data.rs,
  projectile_fields.rs, snbt.rs, and the cushions /
  structure-mob / saved-tag tests; the movement-predicate
  `fall_distance` JSON field is untouched (a separate item).
  Phantom: `AX`/`AY`/`AZ` + `Size`, anchor defaulting to
  BlockPos.ZERO. PrimedTnt: no owner saved. Bucket variants
  ride inside `bucket_entity_data` (axolotl `Variant`, salmon
  `type`, tropical fish `BucketVariantTag` packed) — the
  separate variant components do not exist in the 1.21.4
  registry and their reads are gone. `getBooleanOr` does not
  exist in 1.21.4 (all sites now `getBoolean`). Item 155
  (`mobs/vex.rs`): the plain-field owner form, jar-verified
  (Vex saves BoundX/BoundY/BoundZ + LifeTicks only; no
  `EntityReference` class). No-change, jar-verified: mooshroom
  `Type`, wandering trader `wander_target`, dolphin, piglin,
  ominous spawner, warden anger/listener, the NeutralMob
  AngerTime/AngryAt and AgeableMob Age/ForcedAge forms, and all
  the getXOr/codec/playSound/isDay/snapTo/putUUID reshapes.
  Tests: 4 new list-form round-trip tests (zombie gear, drop
  chances, armor stand, salmon/tropical buckets); the horse,
  wolf, TNT, axolotl, spawn-egg, phantom, raider, mannequin,
  armor-stand, advancement and item-command tests converted to
  the list form. NOT in this item, with jar evidence recorded
  for their own items: finalize.rs (item 117) — 1.21.4's
  `saves_variant_name` is Wolf/Cat/Frog only (chicken writes
  only EggLayTime, cow/pig write no variant key) and
  `sound_variants` is empty for all species (0 `sound_variant`
  strings in the jar); beehive.rs (item 79) —
  IGNORED_BEE_TAGS is the 30-entry 1.21.4 set
  (ArmorDropChances/HandDropChances, `FallDistance`); phantom
  goal code (item 401); the tropical-fish bucket tooltip
  (item 22). The nautilus's `equipment`-compound saddle save
  (mobs/nautilus.rs) is dead code — no nautilus in 1.21.4, no
  spawn path — left as the 1.21.10 port left unremoved species.
  SpawnData's `equipment` field is real in 1.21.4
  (`EquipmentTable`), so spawner.rs is untouched. Rank 2:
  57/348.

  Batch-2 compile follow-ups (same commit): the batch-2
  `cargo check` never reached the horizon-client lib — it died
  on the GAME_RULES length error in horizon-client-lib first —
  so the batch-3 check surfaced 19 missing-method errors from
  the system removals, all fixed to the 1.21.5 form. The deleted
  `game/ghasts.rs` held the generic entity-collision helpers, so
  `game/entity_collisions.rs` is restored from 1.21.5 (Shulker/
  boat only, no ghast arms) with its test; `game/locator_bar.rs`
  held `own_uuid`, restored into `game/names.rs` with its `uuid`
  helper; the `close_dialogs` (5), `dialog_scroll` (1),
  `track_experience_display` (2) calls are gone with the
  `experience_display` field, and `Remote::player_entity` (whose
  only caller was the deleted locator bar) is gone too. The
  /version command's `version_lines` went with cmd_version, so
  the F3 DumpVersion debug action and its key binding are gone
  (ACTIONS 20 -> 19). `AXOLOTL_VARIANTS` is test-only now.
  Full `cargo check -p horizon-client` green (EXIT 0, 0 errors)
  after these fixes; the check's one warning (the dead
  `Remote::player_entity`) was fixed by removing the method, and
  the next batch's check re-verifies the tree warning-free.

  Batch 4 (items 12 + 13). Item 12 (game.rs): DecoratedPotBlock
  and TripWireBlock `playerWillDestroy` setBlock flags 260 -> 4
  (1.21.4's `UPDATE_NONE` is 4 — `Block` has no
  `UPDATE_SKIP_BLOCK_ENTITY_SIDEEFFECTS`, and the Rust
  `set_block` skips block-entity side effects at flag 256, so
  260 changed behaviour); ComposterBlock `handleFill` particle
  bounds 0.1875/0.625 -> 0.13124999403953552/0.737500011920929
  (game/composter.rs, which has no report item of its own; the
  three gaussians are x0.02 in both versions, the diff's reorder
  was a local-renumbering artifact). No-change, jar-verified:
  BeehiveBlock `playerWillDestroy` (`preventsBlockDrops` is
  `instabuild`, same as `isCreative`) and `playerDestroy`
  (`Containers.updateNeighboursAfterDestroy` is a trivial
  wrapper), ShulkerBoxBlock `playerWillDestroy`, the
  Scaffolding/Shulker shape renames, `handleKeybinds`/
  `startUseItem` (`hasInfiniteMaterials` == `isCreative`),
  MouseHandler (same scaled-pos math), `storeGameTypes` (same
  NBT), the RespawnConfig message rule (send iff the point
  moved, both versions), the Entity-vs-Player overloads
  (levelEvent, mayInteract, BucketItem, candle extinguish),
  `loadAndSpawnParentVehicle` (RootVehicle contains-check),
  `openItemGui`, `getSituationalMusic`, the IDE-only world
  screen branch. Item 13 (game/remote.rs):
  `handleMinecartAlongTrack` gains the 1.21.4
  `isControlledByLocalInstance` early return (the local player
  riding the cart skips the server's steps; 1.21.5 dropped the
  guard); the game-event type is `ARROW_HIT_PLAYER` (id 6 in
  both jars — the 1.21.5 rename is name-only). No-change,
  jar-verified: the game/sound event overloads, the
  isLocalInstanceAuthoritative/isControlledByLocalInstance and
  setSelectedSlot renames, the login chat-index reset, the
  portal/spinning effect rename (the respawn intensity copy is
  unmodeled), the chat-signature error path (unmodeled),
  `hasInfiniteMaterials` == creative, the effect renames, and
  the pose-priority flip (FALL_FLYING-first vs SLEEPING-first
  is unobservable — the states are mutually exclusive, and the
  Rust guest pose already checks gliding before swimming).
  Routed with jar evidence: the player respawn flat save form,
  `enteredNetherPosition`, no `raid_omen_position`, forced
  chunks via `chunks.dat` (`Forced` long array — no
  `TicketStorage` in 1.21.4) and the ticket lifecycle (item 23);
  the `canDestroyBlock` tool-component check (item 119 —
  1.21.4's Tool record has no `canDestroyBlocksInCreative`);
  KeyboardInput normalization + the sprint 0.8 impulse
  threshold (item 15 batch — `keyboard_direction` normalizes,
  the 1.21.5 form; 1.21.4 stores raw impulses, needs forward
  >= 0.8 to start a sprint on land, > 1e-5 underwater);
  `Player.aiStep` swing/head placement and the fall-damage
  formula (item 15); SetPassengers on `startRiding` (riding
  items); `tickChunk` thunder (horizon-world tick item);
  `Raids.tick()` (item 21); the scaffolding placement
  collision (shape item — the Rust `is_placement` flag is
  always false, so behaviour already matches); the
  `onProjectileHit` pot crack 260 -> 4 in creatures.rs:5090
  (item 17). Rank 2: 59/348.

  Batch 5 (items 14 + 212). Item 14 (entities.rs): the
  lava-cauldron contact test is the 1.21.4
  `isEntityInsideContent` form — feet below the lava surface
  (0.9375) and top above a quarter block, no horizontal test
  beyond the cell overlap, the rim and feet do not count (the
  1.21.6 filled-shape test, its `VoxelShape`/portal-travel
  imports and the `OnceLock` shape are gone; the 1.21.5
  lava-cauldron test is carried, adapted to 1.21.4);
  `LavaCauldronBlock.entityInside` is lavaHurt only — the hurt
  itself ignites for 15 s, there is no `lavaIgnite` in 1.21.4.
  The orb-spawn system drops everything 1.21.6 added:
  `ExperienceOrb::unstuck` and its tests are gone (no
  `unstuckIfPossible` in 1.21.4), `spawn_orbs`/`add_orb` are
  the plain 1.21.5 forms (`spawn_orbs_toward`/`add_orb_toward`/
  `unstuck_orb` removed; yaw is a `nextDouble`, the motion's
  x/z factors the widened `0.2F`/`0.1F` literals), the stuck
  orb's `moveTowardsClosestSpace` nudge is unconditional (the
  1.21.6 next-move-collision guard removed), `/summon
  experience_orb` spawns exactly at `pos`, and the thrown
  experience bottle awards plain (its `splash` loses the
  `motion` parameter; both callers updated). Orb NBT: Health
  reads `getShort` with no default — 0 when absent, not 5
  (jar-verified; 5 is the 1.21.5 `getShortOr` form). Also:
  `AttributeMap.save` (not `pack`), and the happy-ghast
  comments are gone from four sites. No-change, jar-verified:
  the remote packet-velocity application at spawn
  (`SetEntityMotion` overwrites with the same value before the
  entity's first tick, so applying the packet velocity is
  unobservable), `collidedWithShapeMovingFrom`/`collide`/
  `isColliding` (API-only), the in-memory `fall_distance` f64
  (its save form was ported in batch 3), and the 1.21.5-only
  methods absent from 1.21.4 (`snapTo`, `propagateFallToPass-
  engers`, `lavaIgnite`, `followNearbyPlayer`,
  `scanForMerges`, `applyComponentsFromItemStack`, and the
  rest — none is modelled in the 1.21.4 tree). Item 212
  (game/item_entities.rs): its module carries the orb-spawn
  port above; its own two methods are no-change —
  `FurnaceResultSlot.checkTakeAchievements` (the `onCraftedBy`
  overload gains the Level arg; unmodelled in Rust) and
  `SculkBlock.attemptUseCharge` (full bytecode identical
  except the `playSound` Entity-vs-Player overload).
  Routed with jar evidence: `ServerEntity.sendChanges`
  (lan/tracking — no rank-2 item covers it); the `baseTick`
  frozen handling (items 187/15); `causeFallDamage`/
  `checkFallDamage` (landing.rs, items 126/296);
  `applyEffectsFromBlocks` (items 15/126/36/66);
  `checkInsideBlocks` (items 15/36); `extinguishFire`/
  `igniteForTicks` (items 55/15/36/160); `isInLiquid` (item
  126); the `teleportTo` pitch clamp [-90, 90] (item 42);
  `NameTagItem`'s `canBeNameTagged` check (item 17);
  `HoneyBlock` entityInside/fallOn (item 52);
  `placePortalTicket` (likely unmodelled in the client).
  `cargo check -p horizon-client` green (EXIT 0, 0 errors).
  Rank 2: 61/348.

  Batch 6 (item 15). Item 15 (motion.rs + the input/sprint/fall
  system, which has no rank-2 item of its own): the input model is
  1.21.4's — `KeyboardInput.tick` stores RAW impulses, so
  `keyboard_direction` returns them unnormalized (1.21.5
  normalizes into `moveVector`; the normalization in 1.21.4
  happens later, in `moveRelative`'s `getInputVector`, which the
  Rust already implements), and `modify_input` drops the
  square-movement correction (`square_movement` deleted;
  1.21.5's `modifyInput` only — 1.21.4 has no `modifyInput`, the
  0.2 item-use and sneaking multipliers apply directly to the
  impulses in aiStep and the 0.98 damping lives in
  `LivingEntity.aiStep`). Movement-equivalence checked by hand:
  raw + no-square reproduces vanilla 1.21.4 exactly in every
  case (full diagonal, item-use, sneak, straight); the old
  normalized + square form matched too for keyboard input, but
  boosted analog diagonals. Sprint is 1.21.4's
  `LocalPlayer.aiStep`: the start threshold is
  `hasEnoughImpulseToStartSprinting` (0.8 on land, any forward
  impulse underwater, on the raw impulse — a half-pushed stick
  no longer starts a sprint on land), the stop block keeps
  `hasForwardImpulse` (> 1e-5), `canStart` forbids fall-flying
  outright (1.21.5 allows it underwater), the start block needs
  the ground (the vehicle's `onGround` for a passenger, threaded
  through `sprint_input_tick_riding` from `driven_mount`) or
  water — no mid-air sprint start (1.21.5 drops the gate) — the
  sprint key also starts a surface swim, the double-tap window
  resets on shift and item use but NOT on backward (the
  backward reset is 1.21.5's), and the fresh-press edge is the
  previous tick's `hasEnoughImpulse`. Three new sprint tests pin
  the threshold, the ground gate and the backward rule. Fall
  damage is 1.21.4's ceil form: `calculate_fall_damage`
  (living_fall.rs) computes `(fall - safe) * multiplier` in
  float, times the fall-damage multiplier, ceiled (1.21.5
  floors `calculateFallPower`); the landing-particle power in
  living_landing.rs ceils too, with the magnitude in float
  (0.2 + power / 15, capped 2.5). A fall-flying player glides
  on past a climbable (1.21.5's `travelFallFlying` climbable
  early-out removed). `checkInsideBlocks` axis order follows
  the RESOLVED move in all three sweep sites (motion.rs,
  entities.rs `orb_lava_contacts` — its `wanted` parameter is
  gone — falling_block.rs `inside_blocks`): 1.21.4 traverses
  straight-line (`boxTraverseBlocks`) and the resolved delta is
  the closer approximation; 1.21.5's `axisStepOrder` takes the
  resolved delta explicitly. Cauldrons: the layered short-circuit
  is carried (entityInside fires on sweep overlap; the content
  check lives in `layered_cauldron_inside`, now taking the
  content top and applying `isEntityInsideContent` — feet below
  the top, head above a quarter block), `getContentHeight`
  (not `getPixelContentHeight`), and the lava-cauldron comment;
  the nether-portal visual-slab test is NOT carried — jar-verified,
  1.21.4's NetherPortalBlock has no
  `getEntityInsideCollisionShape` override (added in 1.21.5),
  so portals fire on whole-block overlap, as the tree already
  does. Same jar check: the base shape is `Shapes.block()` in
  both versions; only end portals and tripwires override it in
  1.21.4 (powder snow's override is 1.21.5's too). The
  `was_on_fire` snapshot in `applyEffectsFromBlocks` is NOT
  carried — 1.21.5-only; 1.21.4 sets -20 whenever not inside
  fire/lava, alive, with `remainingFireTicks <= 0`, which the
  tree's `apply_fire_contact` already does. `AttributeMap.save`
  (not `pack`) in two comments. Drowning: baseTick spawns the
  eight bubbles directly in 1.21.4 (no entity event 67, no
  `makeDrownParticles`); the `MotionEvent::Drown` doc says so
  now. No-change, jar-verified: `MobEffectInstance` save/load
  delegate to the CODEC in 1.21.4 too (the lowercase effect-NBT
  shape stands; the `readAdditionalSaveData` difference is the
  Java call shape only); `LivingEntity.tick`'s run/oRun and
  `animStep` (animation, unmodelled in motion.rs);
  `onAttributeUpdated`'s SCALE branch (1.21.4 refreshes
  dimensions in `tick` instead — same net behaviour);
  `getItemBySlot`/`setItemSlot` (abstract in 1.21.4, concrete
  via EntityEquipment in 1.21.5 — API-only);
  `getKillCredit`/`getLastHurtByMob`/
  `resolvePlayerResponsibleForDamage` (EntityReference vs
  direct fields — same behaviour); `pushEntities` (same
  behaviour, `getEntities` vs `getPushableEntities`);
  `handleRelativeFrictionAndCalculateMovement` (the powder-snow
  check reads the block state directly in 1.21.4, same
  behaviour); `causeFallDamage` (super first, then damage, in
  both — only float/double signatures differ); `breakItem`
  (BREAK_SOUND component vs `getBreakingSound` — same sound);
  `collectEquipmentChanges` (per-slot fields vs the map — same
  behaviour); the MobEffects renames (MINING_FATIGUE/
  DIG_SLOWDOWN, RESISTANCE/DAMAGE_RESISTANCE, JUMP_BOOST/JUMP);
  `tickEffects` (one `tick` vs the tickServer/tickClient
  split); `travel`/`travelRidden` (the
  `isControlledByLocalInstance` early return and the no-arg
  `travelFallFlying` — same values); `LayeredCauldronBlock.
  entityInside` (bytecode-identical); the aiStep
  lerp/InterpolationHandler difference (interpolation is
  unmodelled in motion.rs in both trees). Pre-existing gaps in
  BOTH trees, not port differences: the run-stop's
  `(isInWater && !isUnderWater)` term is missing from
  `should_stop`; blindness is not modelled in sprint
  `Conditions`; the aiStep-start `shouldStopSprinting` check
  (fall-flying/blindness/moving-slowly/using-item) is not
  modelled in the sprint phase. Routed with jar evidence:
  `canUseSlot` (base false in 1.21.4, true in 1.21.5; Player
  overrides it in 1.21.4) and the dispenser `mob_can_use_slot`
  default (item 39); `canFreeze`'s BODY-slot check — leather
  horse armor protects in 1.21.4 (item 187, mobs/ai.rs);
  `getItemBlockingWith`/`hurtServer` shield blocking
  (BLOCK animation + 5-tick delay; `hurtCurrentlyUsedShield`;
  event 29 broadcast) and `entityEventForEquipmentBreak`
  (7 slots, no saddle/68) (item 126); the mob fall-damage
  formula in `mobs/living.rs` (same floor -> ceil port as the
  player path) (item 126); `onEquipItem`/`releaseUsingItem`/
  `onEffectAdded`/`onEffectsRemoved` (items 126/55);
  `handleEntityEvent` 29/30 shield sounds and the missing 67/68
  cases — the LAN event-67 broadcast in lan_actions.rs (item 28)
  and creatures.rs (item 17) should send particles instead, and
  remote.rs's event-67 handling (item 13, closed) is dead in
  1.21.4 play; the swing/head placement (1.21.5's Player.aiStep
  calls `updateSwingTime` and sets `yHeadRot = yRot`; 1.21.4
  does neither — swing advances in `LivingEntity.tick`) (item
  164); `Player.attack`'s sweep ordering (knockback + hurt
  before the server check in 1.21.4; `hurtServer` inside it in
  1.21.5) (item 13, closed — check whether batch 4 covered
  it); the `makeDrownParticles` doc names in combat.rs (item
  210) and mobs.rs (item 55). Follow-up hunks in closed modules:
  the axis-order fix in entities.rs (item 14) and the
  `modifyInput` doc in game.rs (item 12).
  `cargo check -p horizon-client -p horizon-world
  -p horizon-client-input --tests` green (EXIT 0, 0 errors;
  the lib check green too). The --tests run caught two
  test-compile errors from earlier batches (fixed above);
  batches from here run the --tests check, not just the lib.
  Rank 2: 62/348.

  Batch 7 (items 16 + 17 + 18). Item 16 (game/game_tests.rs):
  NO-CHANGE on all three tree-diff hunks. The 1.21.4 jar has no
  TestInstanceBlock classes at all (1.21.5 has 10): 1.21.4's
  game-test framework is the structure-block model
  (`GameTestInfo` with `TestFunction` + `StructureBlockEntity`;
  `prepareTestStructure` returns GameTestInfo and calls
  `StructureUtils.prepareTestStructure`), while 1.21.5's is the
  test-instance-block model (`GameTestInstance` +
  `TestInstanceBlockEntity`, `createTestInstanceBlock` +
  `placeStructure`). The 1.21.4 tree's runner is therefore a
  1.21.5-form system on 1.21.4 data, but it is safe: no panic
  path was found (`set_test_block_state` returns early when
  `default_state` is None; `prepare_test_structure` inserts a
  default TestInstanceBlock block entity when absent), and the
  runner is a dev-only subsystem (`/test`), not player
  gameplay. `track_and_start_runner` (16 lines, 1.21.5-only) is
  dead in 1.21.4 — the 1.21.4 tree's test_instances.rs has no
  test-instance-block action packet and no `track_and_start`
  call, and the 1.21.4 test calls `track_and_start_batches`
  directly — so it is not added. `prepare_test_structure`'s
  `bool` return (false when the block is gone; both callers
  check it) fits 1.21.4's structure-block model, where the
  block can be absent; 1.21.5's `()` form requires the block
  entity. `say` stays `say` (1.21.5 renamed it `say_report`
  because `Game::say` is the narrator's; the 1.21.4 tree has
  no `Game::say`). A rewrite of the runner to the
  structure-block model is a separate job, recorded as a known
  limitation, not a behaviour-port item.

  Item 17 (game/creatures.rs): PORTED. The ravager roar
  broadcast (event 69) is gone: neither 1.21.4's nor 1.21.5's
  Ravager broadcasts it (`roar()` has no broadcastEntityEvent;
  `handleEntityEvent` handles only event 4), and 1.21.4's
  `Ravager.aiStep` fires the roar from the entity's own tick
  (`roarTick == 10`). The creatures.rs handler skips the LAN
  broadcast for event 69; remote.rs (item 13 follow-up) gates
  the event-69 handler on 1.21.6+ servers and fires the roar
  client-side when `anim_ticks == 10` for 1.21.5-or-older
  servers (ravager.rs, item 107 follow-up, comments only:
  AttributeMap.pack -> save, roar doc, event-69 comment). The
  wolf armor shear gate is the 1.21.5 form: 1.21.4's
  `Wolf.mobInteract` shears branch has NO sneak check — shears
  + owned + wearing body armor + (creative if the armor has
  PREVENT_ARMOR_CHANGE) — so the `!sneaking` gate became
  `(creative || !binding_curse)`. `shear_saddle` is deleted
  (mounts.rs: `Mob.canShearEquipment` exists in neither 1.21.4
  nor 1.21.5 — a 1.21.6 addition), and the
  PLAYER_SHEARED_EQUIPMENT trigger fire is deleted (the trigger
  exists in neither version's `Criteria`). The spawn egg on a
  spawner is the 1.21.5 form (type only): 1.21.4's
  `SpawnEggItem.useOn` calls only `Spawner.setEntityId(type,
  random)` — no data, no op check — so `spawn_egg_on_spawner`
  returns a String (block_entity_item.rs, item 129 follow-up),
  `use_spawn_egg_on_spawner` drops its data param
  (game/spawner.rs, item 152 follow-up), `set_entity_data`
  leaves the spawner trait + impl + its test (mobs/spawner.rs,
  unowned module), `override_entity_data` leaves the trial
  spawner trait + impl (mobs/trial_spawner.rs, item 140
  follow-up), and the dispenser.rs call site takes the 1-arg
  form (it was still calling the old 2-arg signature — a
  compile error caught before the check). The decorated-pot
  crack is flags 4, not 260 (routed from batch 4; 1.21.4's
  `DecoratedPotBlock.onProjectileHit` uses `iconst_4`). The
  LAN drowning broadcast is particles, not event 67 (routed
  from batch 6; 1.21.4's `LivingEntity.baseTick` spawns the
  eight BUBBLE particles directly): creatures.rs drops the
  `lan_entity_event(67)` call, combat.rs's `drown_particles`
  gains a `host: Option<&LanHost>` param and sends each bubble
  to LAN (item 210 follow-up), and lan_actions.rs's guest
  Drown handler calls it with `Some(host)` (item 28
  follow-up); the `DROWN_EVENT` const stays for remote.rs's
  1.21.5+ receive handler. The spawn-egg offspring takes only
  the egg's custom name, after the add (1.21.4's
  `spawnOffspringFromSpawnEgg`: `moveTo` then
  `addFreshEntityWithPassengers`, then CUSTOM_NAME ->
  `setCustomName`; 1.21.5's `snapTo` is byte-identical to
  1.21.4's `moveTo` — a pure rename — and its
  `applyComponentsFromItemStack` puts all components on before
  the add). The breeding coin is drawn from the mother's
  random, and only for Fox/Cat/Wolf: 1.21.4's
  `Chicken.getBreedOffspring` is create-only (no draw, no
  variant), Pig/Cow likewise, while Fox/Cat/Wolf draw
  `this.random.nextBoolean()`; the variant-pick branch now
  covers Fox only (Pig/Cow/Chicken have no variants in
  1.21.4). NO-CHANGE, jar-verified: the potion splash centres
  on the hit location (1.21.4's `ThrowableProjectile.tick`
  calls `setPos(hitResult.getLocation())` before
  `hitTargetOrDeflectSelf`, identical in 1.21.5 — the 1.21.4
  tree is right and the 1.21.5 tree's `p.previous` form
  contradicts BOTH jars, a 1.21.5-port deviation flagged to the
  parent, out of scope here); the guardian's
  `isInWaterOrBubble` vs `isInWater` (the Rust's block facts
  classify bubble columns as water, so `in_water` is already
  true there); the dragon fireball's `setPotionDurationScale
  0.25f` (1.21.5-only, unobservable — the cloud's only effect
  is instant HARM, which has no duration); the thrown egg
  (1.21.4's onHit is `setAge` + `moveTo` + add, no variant);
  the egg's PROJECTILE_SHOOT_POWER (1.5f in both — the
  `use()` difference is literal-vs-constant only);
  `EntityType#by` (string + parse + getOptional vs CODEC —
  same result); `updateCustomEntityTag` (1.21.4 takes a Player
  param with a null check, 1.21.5 a LivingEntity with an
  instanceof check — same op-only behaviour in every path the
  Rust models; the dispenser path is identical in both trees,
  a shared gap, not a port difference);
  `DistanceManager#inEntityTickingRange` (a pure rename:
  `tickingTicketsTracker`/`TickingTracker` vs
  `simulationChunkTracker`/`SimulationChunkTracker`);
  TurtleEggBlock's randomTick (snapTo -> moveTo rename),
  fallOn (double vs float — API-only) and decreaseEggs
  (playSound overload); AmethystBlock, Parrot, WitherBoss,
  EnderpearlItem, SnowballItem, WitherRoseBlock (playSound
  Entity-vs-Player overloads only). DEFERRED to batch 8 (the
  leash-system port, unowned): the LeashBreak `sound` field —
  1.21.4's `Leashable` is the LEGACY system
  (`legacyElasticRangeLeashBehaviour`, 10.0/6.0 distances, 0.4
  pull, identical to 1.21.5; `dropLeash` plays no sound),
  while the 1.21.4 tree carries the 1.21.6 Wrench/spring
  system (SNAP=12, `spring`, `elastic_pull`, quad-leash APIs,
  `HolderState.center`), and the creatures.rs hunk cannot be
  done in isolation (the event shape is shared with leash.rs
  producers). Routed: the potion splash's dead `age` param
  (item 26, potions.rs); chicken/pig/cow variant finalize
  (empty registries in 1.21.4 — item 117, finalize.rs);
  `leads.rs` quad-leash call sites (batch 8 / item 463, rank
  3). The 1.21.5 tree's breed has no chicken-variant
  inheritance (the 1.21.5 jar has it) — a second 1.21.5-port
  deviation, flagged to the parent.

  Item 18 (mobs/tame.rs): NO-CHANGE except one comment carry.
  Wolf sound variants do not exist in 1.21.4 (no
  `wolf_sound_variant` registry data in the 1.21.4 jar;
  1.21.5 has sad/puglin/grumpy/...): `getHurtSound` returns
  WOLF_HURT directly (the species table's
  "entity.wolf.hurt" — correct), `defineSynchedData` uses the
  WOLF_VARIANT registry with a RED default collar (the Rust
  defaults to `tame::RED` when CollarColor is absent), and
  `Wolf#get` (the 1.21.5-only component accessor) has no Rust
  equivalent. `getTexture` is the 1.21.4 direct form
  (`tameTexture`/`angryTexture`/`wildTexture` — the Rust's
  `tame::texture` builds `entity/wolf/{file}{_tame|_angry}
  {_baby}`; 1.21.5's assetInfo resolves to the same paths).
  `readAdditionalSaveData` reads "variant" as a string and
  "CollarColor" as an int (entity_nbt.rs does both, with the
  RED default). `tick`'s `isInWaterRainOrBubble` collapses to
  the Rust's `in_water || in_rain` (bubble columns are water
  in the block facts). The one tree-diff hunk, a test comment,
  takes the 1.21.5 form: vanilla writes `AngerTime`/`AngryAt`
  in both versions. ROUTED (found here, owned elsewhere): the
  sound-variant assignment is wrong for 1.21.4 in TWO places
  — finalize.rs's `sound()` (item 117) assigns
  `Some(r.random())` unconditionally, and `random()` returns
  0 for an empty registry, so every 1.21.4 wolf gets
  sound_variant 0 = "angry" (the hardcoded list is sorted
  alphabetically: angry, big, classic, ...); spawning.rs's
  `finalize` (item 188) assigns from the HARDCODED list length
  (`sound_variants(species).len()`), so every spawned wolf/pig/
  cow/chicken/cat gets a random 1.21.5 sound variant. The fix
  (don't assign when the version has no such registry) and the
  entity_nbt.rs sound_variant save/load question (item 11 is
  closed; the save writes "sound_variant" whenever the species
  has a hardcoded list, defaulting to "classic") belong to
  items 117/188.
  `cargo check -p horizon-client -p horizon-world
  -p horizon-client-input` green (EXIT 0, 2m44s; the first run
  failed on a missing `LanHost` import in combat.rs, fixed)
  and the --tests check green (EXIT 0, 6m19s, 0 errors). The
  nine rustfmt-checked leaf files are clean (ravager.rs,
  mounts.rs, block_entity_item.rs, mobs/spawner.rs,
  mobs/trial_spawner.rs, combat.rs, lan_actions.rs,
  dispenser.rs, tame.rs); creatures.rs, remote.rs and
  game/spawner.rs are module roots and were not formatted.
  Rank 2: 65/348.

  Batch 8 (the leash-system port, unowned — deferred from item
  17; closes no numbered item). 1.21.4's `Leashable` is the
  LEGACY system, identical to 1.21.5's (jar-verified:
  `legacyElasticRangeLeashBehaviour`, LEASH_TOO_FAR_DIST=10.0,
  LEASH_ELASTIC_DIST=6.0, 0.4 pull; `dropLeash` plays no
  sound), while the 1.21.4 tree carried the 1.21.6 Wrench/
  spring system (SNAP=12, `spring`, `elastic_pull`, quad-leash
  APIs, `HolderState.center`). Carried the 1.21.5 legacy system
  wholesale: mobs/leash.rs, game/boat_leads.rs and
  game/leads.rs copied from the 1.21.5 tree (version mentions
  in comments adapted 1.21.5 -> 1.21.4; rustfmt drift in the
  three copied files cleaned — 9 small hunks). `MobEvent::
  LeashBreak` loses its `sound` field (mobs.rs, item 55
  follow-up) and `break_leash` loses its third arg
  (creatures.rs handler + bucket call — completing item 17's
  deferred hunk — and the call sites in beehives.rs,
  boat_passengers.rs, copper_golems.rs, knot_identity_tests.rs,
  llamas.rs, minecarts.rs, nautili.rs, sulfur_cubes.rs).
  equine.rs and camel.rs gain `handle_leash_at_distance`
  (the 1.21.5 form, distance > 6.0 gate inside) in place of
  the dead `on_elastic_leash_pull` (its only caller was the old
  leash.rs). Two methods the old leads.rs had are 1.21.6-only
  and were removed with their call sites, jar-verified:
  `drop_all_leash_connections` (neither 1.21.4's nor 1.21.5's
  `FireworkRocketItem.use` calls it — the leash-drop block
  leaves boost_firework) and `forget_untied_knots`
  (`notifyLeasheeRemoved` exists in neither version — a knot
  outlives its last lead, so lan_entities.rs drops the call).
  Dead code the port left behind is removed: `Boat.leash_spin`
  (`LeashData.angularMomentum`) and `Boat.in_liquid`
  (for `Leashable.angularFriction`) — neither exists in the
  1.21.5 tree's boats.rs. Routed: equine.rs's stand-counter
  hunks (counter 1 vs 20, counting up vs down — a separate
  behaviour difference, not leash) to item 205; the chest-boat
  sneak hunk in boats.rs (1.21.4's `AbstractChestBoat.interact`
  checks `isSecondaryUseActive` first, matching the 1.21.5
  form) to item 57; leads.rs's carry to item 463 (rank 3 —
  the quad sites are gone, the rank-3 agent verifies the rest);
  camel.rs's leash hunk to item 395 (rank 3 — it was camel.rs's
  only tree diff). `cargo check -p horizon-client
  -p horizon-world -p horizon-client-input` green (EXIT 0,
  0 warnings) and the --tests check green (EXIT 0, 7m18s).
  Rank 2: still 65/348 (batch 8 closes no numbered item).

  Batch 9 (items 19 + 20 + 21). Item 19 (game/native_hooks.rs):
  NO-CHANGE, comment-only carry. The tree diff is 46 comment
  lines naming 1.21.6 features (the dialog system, the locator
  bar, the `now_playing` toast, the happy ghast) that do not
  exist in 1.21.4; the 1.21.5 tree's comments are correct for
  1.21.4, so the file was copied wholesale (12,786 lines, no
  code diff). The jar diffs are API-only or modelled elsewhere:
  `destroyBlock`'s `canDestroyBlock` vs `canAttackBlock` (item
  119, already routed); the BLOCKS_ATTACKS component in
  `getUseAnimation`/`getUseDuration`/`use` (items 126/55);
  the enderpearl `unRide` (already modelled — `stop_riding`
  before the teleport in creatures.rs); `handleBlockBreakAction`
  / `useItemOn` / `BlockItem#canPlace` / `Item#hurtEnemy` /
  `Level#playSound` (overload and accessor reshaping, same
  behaviour).

  Item 20 (game/minecarts.rs): PORTED. `prime_cart` and
  `explode_cart` drop their `tnt_explodes` gamerule gates:
  `RULE_TNT_EXPLODES` does NOT exist in 1.21.4 — it was added
  in 1.21.5 (verified: the 1.21.4 jar has no such field in
  `GameRules`, and no references to it in `MinecartTNT`,
  `PrimedTnt`, `TntBlock`, `ServerLevel` or `Explosion`, while
  1.21.5's `TntBlock` has 3 and its `MinecartTNT.primeFuse` /
  `explode` check it). In 1.21.4 a TNT minecart primes and
  explodes regardless of any gamerule. Two comment carries:
  `CompoundTag`'s (not `ValueInput`'s) numeric getters, and
  `cart_lava_tick` no longer names `lavaIgnite` (absent in
  1.21.4 — the cart takes `lavaHurt` twice and never burns
  because it has no `baseTick`). NO-CHANGE, jar-verified: the
  fall damage (`(fall / 10)^2` at fall >= 3); the display
  offset/block (the Rust returns the default without a custom
  display — the 1.21.4 `hasCustomDisplay` form; display.rs is
  identical in both trees); the destroy fuse (two
  `nextInt(20)` draws); the render `mulPose`, `getDisplay-
  BlockState`, `causeFallDamage` and `primeFuse()` shapes
  (API-only); the `getKnownMovement` NaN guard and the
  interpolation check (unmodelled). SHARED GAP (both trees,
  not a port difference): `hurt_vehicle` passes `fire=false`,
  so a burning projectile never primes a TNT cart — 1.21.4's
  `hurtServer` checks any `Projectile`, 1.21.5's only
  `AbstractArrow`, both requiring on-fire plus a speed
  threshold; the `Projectile` struct has `on_fire()`, so the
  fix is feasible later. ROUTED: the cross-cutting
  `tnt_explodes` removal — the rule is defined in
  gamerules.rs (no item) and read in 13 files (arrows.rs,
  game.rs, chat.rs, creatures.rs, dispenser.rs, explosion.rs,
  sulfur_cubes.rs, tnt.rs, sulfur_cube.rs, save.rs,
  datafix/fixes_c.rs, plus the minecarts.rs sites done here)
  — to item 66 (tnt.rs) as its natural home.

  Item 21 (raid.rs): NO-CHANGE. The tree diff is empty, and
  every jar diff is API-only: `level` as a method parameter
  (1.21.5) vs the `this.level` field (1.21.4) in
  `findRandomSpawnPos`, `playSound`, `spawnGroup`, `tick`,
  `updateRaiders`, `createOrExtendRaid` and `Raids.tick`;
  `raidMap` as fastutil `Int2ObjectMap` vs `java.util.Map`;
  `getNumGroups` returns the same 3/5/7 for EASY/NORMAL/HARD
  (only the switch compilation differs);
  `getOminousBannerInstance` hides the banner patterns via
  `TOOLTIP_DISPLAY.withHidden` (1.21.5) vs
  `HIDE_ADDITIONAL_TOOLTIP` (1.21.4) — same result, and the
  banner item itself is unmodelled in raid.rs;
  `findRandomSpawnPos` is local-slot renumbering plus
  instruction reordering with identical operations and
  constants (0.22f, PI, 32.0f, the same random draws).
  `cargo check -p horizon-client -p horizon-world
  -p horizon-client-input` green (EXIT 0, 1m28s) and the
  --tests check green (EXIT 0, 3m31s, 0 errors). minecarts.rs,
  native_hooks.rs and minecarts/display.rs are rustfmt-clean.
  Rank 2: 68/348.

  Batch 10 (items 22 + 23 + 24). Item 22 (item_tooltip.rs): PORTED.
  Tooltip hiding is the core 1.21.4-vs-1.21.5 system difference:
  1.21.5 uses the TooltipDisplay component, 1.21.4 uses HIDE_TOOLTIP
  / HIDE_ADDITIONAL_TOOLTIP unit components plus per-component
  showInTooltip flags (verified: 1.21.4's DataComponents has both
  HIDE components and has NO WEAPON and NO TOOLTIP_DISPLAY
  component). TooltipDisplay gains a hide_tooltip field, set in
  for_stack from minecraft:hide_tooltip; hides_tooltip() now returns
  hide_tooltip OR the tooltip_display check (kept, since a 1.21.5+
  server can still send it). In 1.21.4 play stacks never carry
  tooltip_display, so the old gating was a no-op; hide_tooltip now
  hides all gated lines (lore, bees, honey, fireworks, jukebox,
  enchantments, dyed, ominous bottle, bundle, instrument, tropical
  fish, map, spawn egg, block state, written book). Comment carry:
  ItemStack.addModifierTooltip (the 1.21.5 form; the tree named the
  1.21.6 ItemAttributeModifiers.Display.Default.apply). NO-CHANGE:
  the EQUIPPABLE equipOnInteract branch and the WEAPON component
  (both 1.21.5-only, unmodelled), PotionContents.addPotionTooltip
  (the Rust's brewing::tooltip builds the same "EffectName II
  (duration)" output), inventoryTick / onCraftedBy (API-only).
  Recorded gaps, not fixed: hide_additional_tooltip (needs the
  appendHoverText/component split the Rust does not model — the
  details fn mixes both); trim, pot decorations, potions and
  attributes do not check shows(); lore ordering (Rust puts lore
  right after the name, 1.21.4 after enchantments/dyed) is a shared
  deviation in both trees.
  Item 23 (save.rs): PORTED. (1) The player respawn point is FLAT
  fields in 1.21.4 — SpawnX/SpawnY/SpawnZ (int), SpawnAngle (float),
  SpawnForced (bool), SpawnDimension (string) — not the 1.21.5
  respawn compound (RespawnData does not exist in 1.21.4; verified:
  no RespawnData class, ServerPlayer saves the flat fields).
  read_player and player_tag rewritten to the flat form; the stale-
  keys list drops the flat keys (plus the 1.21.5 compound,
  defensively). (2) Forced chunks live in data/chunks.dat, not
  chunk_tickets.dat: 1.21.4 has ForcedChunksSavedData (FILE_ID
  "chunks", TAG_FORCED "Forced", a long array of ChunkPos.asLong
  values under data/, no minecraft/ subfolder) and no TicketStorage
  at all (1.21.5 has TicketStorage and no ForcedChunksSavedData).
  load_forced_chunks reads the Forced long array; save_chunk_tickets
  is renamed save_forced_chunks and drops its portal param;
  load_portal_tickets returns empty (portal tickets are memory-only
  in 1.21.4); is_forced_ticket / is_portal_ticket removed; the
  portal_tickets_fixture probe is deleted (its case —
  chunk_tickets.dat persistence — does not exist at this pin; no
  other tree reference); saving.rs call sites updated; the
  PortalTicket doc fixed. The forced-chunks test is rewritten for
  chunks.dat (the foreign-ticket preservation it asserted is a
  1.21.5 TicketStorage behaviour with no 1.21.4 counterpart). (3)
  Legacy numeric dimensions carried: DimensionType.parseLegacy
  exists in BOTH 1.21.4 and 1.21.5, so read_player now maps -1/0/1,
  with the 1.21.5 test carried. (4) The test_instance_block POI
  mapping removed (no test instance blocks in 1.21.4). (5) Conduit
  comments fixed to putUUID/getUUID (the code already wrote/reads an
  int array of four — correct either way). NO-CHANGE:
  world_gen_settings (level.dat only — correct for 1.21.x),
  FallDistance (Float — correct). ROUTED: needs_file_fixes / the
  file_fixing backup prompt to item 400 (title.rs, rank 3) — the
  1.21.4 tree's title.rs has no file_fixing at all, so the fn
  belongs with that feature; enteredNetherPosition persistence
  (modelled in-memory as adv.marks.nether, not persisted to
  player.dat — a cross-module follow-up; the field is a 3-double
  Vec3 list in 1.21.4's player.dat). The playerdata.rs
  PrepareSpawnTask comment now names SpawnAngle (not
  respawnData.yaw/pitch).
  Item 24 (game/stations.rs): PORTED. The tree diff is empty and the
  jar diffs are the 1.21.5 TooltipDisplay refactor moving tooltip
  code from item classes into components — output unchanged (spot-
  checked: 1.21.4's WrittenBookItem.appendHoverText emits
  book.byAuthor + generation, identical to 1.21.5's
  WrittenBookContent.addToTooltip; Item.appendHoverText is a no-op
  in both jars, so the smithing-template super-call difference is
  nil; FireworkExplosion is the addShapeNameTooltip helper vs
  inline, same output). The one real difference: JukeboxPlayable is
  a {song, show_in_tooltip} record, and 1.21.4's addToTooltip checks
  showInTooltip itself (1.21.5 leaves it to TooltipDisplay).
  jukebox_line (item_tooltip.rs) now reads the song from the record
  and honours show_in_tooltip, and jukebox.rs jukebox_playable is
  fixed the same way — both had misread the record compound as an
  inline song (no description key, so the tooltip line never showed
  and the jukebox lost the song id for an explicitly stored disc;
  plain discs still worked through the item default).
  game/jukebox.rs has no item at any rank, so its fix lands here.
  ItemLore / OminousBottleAmplifier / SuspiciousStewEffects
  addToTooltip are bytecode-identical. Lib check green (EXIT 0,
  2m14s); --tests check green (EXIT 0, 3m14s). Rank 2: 71/348.

  Batch 11 (items 25 + 26 + 27). Item 25 (menu.rs): NO-CHANGE. The
  tree diff is empty. The Inventory compartments difference
  (1.21.4's List<NonNullList> vs 1.21.5's flat items +
  EntityEquipment) is structural only: the Rust models the player
  inventory as a flat 36-slot vec plus separate armor/offhand
  fields, never as compartments, so contains / dropAll / setItem /
  tick all agree in behaviour, and native_inventory_tick(slot,
  selected) is already the 1.21.4 form (1.21.5 passes an
  EquipmentSlot). AnvilMenu's hasInfiniteMaterials vs
  getAbilities().instabuild is the same value (the Rust passes
  creative through; cost >= 40 && !creative). The onCraftedBy
  signature in the result-slot achievements is API-only. MapItem's
  onCraftedPostProcess / scaleMap run on any Level in 1.21.4 and
  require a ServerLevel in 1.21.5 — the Rust runs post-processing
  server-side with no gate, matching 1.21.4.
  Item 26 (game/potions.rs): PORTED. (1) The splash system is
  carried from the 1.21.5 tree, jar-verified against 1.21.4's
  ThrownPotion.applySplash: entities in the potion's box inflated
  (4, 2, 4), distanceToSqr from the potion's position with a 16
  cutoff, full strength for the directly hit entity else 1 -
  sqrt(d2)/4. splash_intensity(from, direct, pos, bounds) replaces
  splash_distance_sqr (the 1.21.6 box-to-box + age-margin form); the
  landed tuple carries from (t.position) instead of age, which also
  resolves the dead-age routing from item 17. ENTITY_HIT_MARGIN
  (0.3) is added to projectiles.rs —
  DEFAULT_ENTITY_HIT_RESULT_MARGIN is 0.3f in both 1.21.4 and
  1.21.5, and computeMargin exists in neither. The creatures.rs
  splash call site passes at as from (the potion's position at the
  hit IS the hit location after setPos — the 1.21.5 tree's
  p.previous there contradicts both jars, as batch 7 found); the
  xp_bottle test call site is updated the same way. (2) The cloud
  particle is carried: the Particle key + load_particle (updateColor
  recolour) + particle_options replace the 1.21.6 custom_particle
  form (both jars use "Particle"). (3) Cloud NBT load defaults are
  1.21.4's plain getInt/getFloat zeros (Radius 0, Duration 0,
  WaitTime 0, ReapplicationDelay 0), not 1.21.5's getOr defaults.
  (4) There is no potion_duration_scale in 1.21.4 — no such
  DataComponent, no cloud field: the load pre-scales a named
  potion's effects by /4 (the mapDuration lambda, plain integer
  division) and sets duration_scale 1.0 so custom effects stay
  unscaled; the save drops the key; the round-trip test is updated.
  (5) The creeper cloud is unscaled: its effects reach the cloud
  through addEffect, i.e. custom effects, which 1.21.4 applies
  unscaled (1.21.5's setPotionDurationScale(0.25) is the 1.21.5
  form). (6) Both duration != -1 guards are removed from
  tick_potion_cloud: 1.21.4 has neither, so an infinite cloud
  discards at age >= wait - 1 and a duration on use makes it finite.
  NO-CHANGE: the InfestedMobEffect snapTo/moveTo rename; the
  MobEffectInstance tick split (the Rust has a single tick); the XP
  bottle's position() vs getLocation() (the same point after
  setPos); the playSound / mayInteract overloads; the
  ThrowablePotionItem factory. ROUTED: the 1.21.6 computeMargin ramp
  also leaks into arrows.rs (item 36), fireworks.rs, fishing.rs and
  trident.rs — their items verify. The 1.21.5 files potions.rs,
  cloud_particle.rs and cloud_nbt.rs were copied wholesale, then the
  1.21.4 edits above applied; rustfmt drift in the two copied leaf
  files cleaned (potions.rs is a module root and was not formatted).
  Item 27 (widely named: network/codec): NO-CHANGE. 20 vanilla
  classes (all anonymous ByteBufCodecs$N inner classes), 153
  methods, 0 named in Horizon; module_status missing-both — no
  module file in either tree, nothing to port. Codec behaviour lives
  in horizon-protocol, mechanically ported for 769. Lib check green
  (EXIT 0, 1m26s); --tests check green (EXIT 0, 1m45s). Rank 2:
  74/348.

  Batch 12 (items 28 + 29 + 30). Item 28 (lan_actions.rs):
  PORTED, and it surfaced two 769 wire-format bugs the mechanical
  layer had missed. (1) The missing-respawn yaw carry: the 1.21.4
  tree now sets the guest's yaw to 0.0 when no respawn block is
  found, as the 1.21.5 tree does (TeleportTransition.
  missingRespawnBlock is byte-identical in both jars). The jar also
  sets pitch 0, but neither tree resets pitch — a shared gap.
  (2) UpdateAdvancements: 1.21.4's packet has exactly four fields
  (reset, added, removed, progress); the trailing showAdvancements
  bool is a 1.21.5 addition, and the 769 codec was reading it,
  desyncing every advancement packet. The show field is gone from
  the codec, from the manual LAN writer (lan/advancements.rs
  ::update loses its show param; hub.rs::flush_advancements loses
  its show param; both callers passed true) and from the toast gate
  in game/advancements.rs (1.21.4 toasts on completion, subject to
  show_toast + toast_allowed). (3) ContainerClick: 1.21.4 sends full
  ItemStacks for the changed slots and the carried item (the map
  codec is SHORT keys + ItemStack.OPTIONAL_STREAM_CODEC); 1.21.5
  sends HashedStack hashes. The codec now uses Slot, remote.rs
  sends to_network stacks directly (hashed_stack deleted, with its
  Hashed + SyncedNames imports), and lan/containers.rs compares the
  claimed full stacks (hashed_matches -> claimed_matches, item +
  count as before). The 769 adapter steps are updated to match:
  step_1_21_4's up-step passes clicks through with id remapping
  instead of hashing, and translate/v1_21_4's test literal is a
  full stack now (its down-step still drops the predictions).
  Hashed/HashedSlot stay in item.rs: the custom-component wire test
  still uses Hashed::create as a comparison vehicle. Verified that
  the tree's own paths encode the internal structs raw
  (net.rs Session::send does w.send(&packet.encode()), no
  translation), so the internal layout must be 769 — the fix is
  required, not optional. NO-CHANGE: the PlayerCommand action
  numbers (1.21.4 has no PRESS/RELEASE_SHIFT_KEY, so 5/6/3/4/1/2
  are right and the crouch test via PlayerInput flags is right);
  the TestInstanceBlockAction / SetTestBlock handlers (correctly
  absent — handleTestInstanceBlockAction is 1.21.5-only); the
  handleMovePlayer awaiting-teleport early return; handlePickItem-
  FromEntity (getEntity, no part resolution); handleSetCreative-
  ModeSlot, handleUseItemOn, tick, tryPickItem (API-only);
  handleBundleItemSelectedPacket (threading boilerplate).
  ROUTED: the structure-block strict flag (a 1.21.5 addition) to
  item 394 (rank 3) — the packet bit is wire-harmless (a 1.21.4
  client never sets bit 8 of the flags byte), but the entity field,
  the edit-screen STRICT toggle and the load_settings param need
  removing there. KNOWN GAP (programme-level, recorded not fixed):
  the older-clients adapter chain (client_session.rs) still applies
  the 770->769 steps at this 769 pin — newer_layout_down runs every
  step for a 1.21.4 peer, so the show-flag strip and the other
  770-layout arms would corrupt 769 payloads; the chain needs
  gating on the pin (or the older-clients feature dropping for
  ported trees). accept_older_clients is off by default and the
  feature is admitted unfinished, so this bites nobody today.
  Item 29 (widely named: nbt): NO-CHANGE. 33 vanilla classes, 145
  methods, 0 named in Horizon; module_status missing-both — the
  same shape as item 27. Item 30 (fox.rs): NO-CHANGE, comment-only.
  isBrightOutside does not exist in 1.21.4, and 1.21.5's form is
  byte-identical to 1.21.4's isDay (!hasFixedTime && skyDarken < 4),
  so the StrollThroughVillageGoal and getAmbientSound differences
  are pure renames. Fox save/load already match 1.21.4 (Type string,
  Trusted as a list of int-array UUIDs, boolean flags); addTrusted-
  UUID's slot behaviour verified against the jar (slot 0 if free,
  else replaces slot 1). Comment carries only: getTrustedEntities
  -> getTrustedUUIDs, addTrustedEntity -> addTrustedUUID, no
  clearTrusted in 1.21.4, TRUSTED_LIST_CODEC -> the manual
  NbtUtils.createUUID list (fox.rs + entity_nbt.rs). CaveVines#use
  is a playSound overload; registerGoals is an ldc/ldc_w
  constant-pool difference. Lib check green (EXIT 0, 2m19s);
  protocol --tests check green (EXIT 0, 3m37s); full --tests check
  green (EXIT 0, 59m34s — slow: the incremental cache had been
  deleted to free quota, and the VM was contending with another
  agent's horizon-prepare). The first --tests run failed E0308 on
  the translate/v1_21_4 test literal (Hashed carried stack); the
  retry hit the disk quota (104G used) and was unblocked by deleting
  the 1.21.4 target/debug/incremental cache (4.3G, regenerable).
  rustfmt: 5 of 7 leaf files clean; remote.rs had one drift hunk in
  the edited region (fixed by hand) and one pre-existing drift hunk
  at 5551 (left alone); tests.rs has one pre-existing drift hunk at
  926 (left alone). Rank 2: 77/348.

  Batch 13 (items 31 + 32 + 33). Item 31 (villager_brain.rs):
  NO-CHANGE. The tree diff is empty and every jar difference is
  the Holder-vs-direct profession/type API (customServerAiStep,
  playWorkSound, registerBrainGoals, setVillagerData,
  wantsToPickUp) with identical behaviour. getTypeName: 1.21.4
  builds entity.minecraft.villager.<profession-path>, exactly what
  Mob::command_name emits. getBreedOffspring was undetermined by
  the differ and was verified by javap: one nextDouble, <0.5 the
  biome's type, <0.75 the mother's, else the father's — exactly the
  Rust form. HarvestFarmland#tick and SecondaryPoiSensor#doTick
  are bytecode-identical. Item 32 (block_entity_nbt.rs): PORTED.
  (1) The loadAllItems slot default is carried: a missing Slot
  reads 0 (getByte), so the 1.21.4 tree's drop of such entries was
  wrong; unparseable stacks drop (orElse(EMPTY)). (2) The
  TrialSpawner comment is carried (TrialSpawnerData exists in
  1.21.4; TrialSpawnerStateData.Packed is the 26.3 form).
  (3) canonical() no longer writes a "components" key at all:
  1.21.4's saveWithoutMetadata MERGES the encoded components into
  the tag itself (the lambda calls tag.merge), while 1.21.5 stores
  them under "components" — so a 1.21.4 save never carries the key
  and neither tree's form was right (the 1.21.4 tree always wrote
  an empty compound; the 1.21.5 tree omitted it only when empty).
  The every_type_has_defaults test now asserts no components key.
  (4) The structure-block "strict" flag is removed from
  canonical() (no strict field in 1.21.4's StructureBlockEntity;
  the entity/UI side stays routed to item 394). NO-CHANGE:
  CompoundTag#merge (same recursive behaviour; no Rust equivalent
  in this module), LockCode#addToTag (encode+put vs store, the same
  "lock" compound — fromTag reads the "lock" key in both versions),
  tryLoadLootTable (codec vs manual reads, same values), the skull's
  lowercase "custom_name" (jar-verified: putString "custom_name"),
  CustomName top-level strings (BaseContainerBlockEntity writes
  them with putString in 1.21.4). KNOWN GAP: canonical() builds
  from scratch, so top-level component keys from a 1.21.4 save are
  dropped rather than merged back. Item 33 (nbt_path.rs):
  NO-CHANGE, comment-only. 1.21.4 has no NumberTag/intValue
  /longValue — the numeric tags extend NumericTag with getAsInt/
  getAsLong — but the semantics are identical (Double getAsInt/
  getAsLong floor, Float getAsInt floors, Float getAsLong
  truncates), exactly the Rust int_value/long_value forms; only the
  doc comments named the 1.21.5 methods. compareNbt (entrySet vs
  getAllKeys), getDataVersion (contains+getInt vs getIntOr) and
  readBlockState (AIR default when Name is missing, same
  properties loop) are API-only with identical behaviour. Lib check
  green (EXIT 0, 2m06s); --tests check green (EXIT 0, 3m08s).
  Both edited leaf files rustfmt-clean. Rank 2: 80/348.
  Batch 14 (items 34 + 35 + 36). Item 34 (animate_tick.rs):
  PORTED. The ambient desert sound system does not exist in 1.21.4
  (no AmbientDesertBlockSoundsPlayer in the client jar, no desert
  sound tags in 1.21.4 data, no DryGrass blocks, DeadBushBlock has
  no animateTick, block.sand.idle in neither sounds.json) — removed
  wholesale, not carried: Animate::Falling loses its sand flag,
  the DeadBush/DryGrass variants and their tick arms are gone, the
  desert_sand_sounds / desert_dry_vegetation_sounds / terracotta
  tables are gone, sand_sound_nearby and
  desert_dry_vegetation_sounds are gone, and block_animate_tick
  loses its orphaned player param (the DryGrass arm was its only
  user). Neither tree's form was right for 1.21.4 (the 1.21.4 tree
  carried the 1.21.6 three-tag system, the 1.21.5 tree the 1.21.5
  single-tag one). NO-CHANGE: the SoundSource AMBIENT-vs-BLOCKS
  differences (eyeblossom, hanging moss, lava/water fluids) are
  unobservable — sound() carries no category; Eyeblossom
  tryChangingState is unmodelled (1.21.5 uses
  CreakingHeartBlock.isNaturalNight, 1.21.4 ServerLevel.isDay with
  the open-check inverted); leaf drip (LeavesBlock
  #makeDrippingWaterParticles, missing-new) is unmodelled; lava/
  water entityInside lives in entities.rs/fluids.rs, not here.
  ROUTED to item 80: the lava randomTick fire gate — 1.21.4 has
  only doFireTick, no RULE_ALLOWFIRETICKAWAYFROMPLAYERS, and the
  Rust models no doFireTick at all (can_spread_fire_around gates
  only on the nonexistent fire_spread_radius_around_player rule,
  default -1 = always); item 82 owns the fire tick's shared gate.
  Item 35 (mesher/fluid.rs): NO-CHANGE, comment-only. The only
  named method is Shapes#blockOccludes [missing-new], which is
  1.21.4 Shapes#blockOccudes renamed (the typo was fixed in
  1.21.5) — 1.21.4 LiquidBlockRenderer.isFaceOccludedByState calls
  blockOccudes with the same logic (empty-shape and full-block
  shortcuts, fluid box vs face occlusion shape). The Rust's
  coverage() implements the test directly from shape boxes; only
  its doc comment named the 1.21.5 spelling. Item 36 (arrows.rs):
  PORTED. (1) The computeMargin ramp is removed — flat
  ENTITY_HIT_MARGIN (0.3) from the first tick; computeMargin
  exists in neither jar (routed from batch 11). (2) The
  powder-snow fire clear is carried: 1.21.4 applyEffectsFromBlocks
  extinguishes via isInPowderSnow, which the arrow's
  apply_block_effects does not model (Extinguish only for water),
  so a direct is_powder_snow(state) arm is added. (3) The
  contain-check no longer zeroes velocity — setDeltaMovement(ZERO)
  after setInGround is a 1.21.5 addition; 1.21.4 only sticks (the
  in-ground branch returns before any move, so the retained
  velocity is inert until the arrow falls out). (4)
  is_pushed_by_fluid now always returns true — AbstractArrow has
  no override in 1.21.4 (the !isInGround override is 1.21.5);
  behaviourally a no-op today, since the stuck-arrow tick returns
  before the fluid tick. (5) The bubble column: the gate is
  isAir above (1.21.4), not the 1.21.5 collision-shape/fluid
  gate, and the velocities are clamped per onAboveBubbleCol
  (drag max(-0.9, vy-0.03), lift min(1.8, vy+0.1)) and
  onInsideBubbleColumn (drag max(-0.3, vy-0.03), lift min(0.7,
  vy+0.06), fall reset). (6) The FireIgnite comment is carried
  (the inline entityInside form; igniteForTicks is a max, matching
  the Rust). NO-CHANGE verified: EnderMan hurtServer (the
  ThrownPotion class rename only); the arrow save/load forms match
  1.21.4's manual NBT (pickup byte ordinal, damage an
  unconditional double, SoundEvent a registry-key string, item/
  weapon via save, item falling back to getDefaultPickupItem);
  ARROW_HIT_PLAYER is already the 1.21.4 name in remote.rs; the
  on-fire flag sync is covered by synced_fire. The tnt_explodes
  refs stay, routed to item 66. Lib check green (EXIT 0, 2m08s);
  --tests check green (EXIT 0, 3m21s). All three edited files
  rustfmt-clean (arrows.rs checked, not reformatted — it declares
  the client_tests file module). Rank 2: 83/348.
