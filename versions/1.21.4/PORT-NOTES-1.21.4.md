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
