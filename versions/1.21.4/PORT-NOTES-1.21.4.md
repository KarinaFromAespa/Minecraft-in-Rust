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
- **`/clone` still accepts a `strict` argument** (batch 16):
  `strict` is a 1.21.5 addition to CloneCommands (0 occurrences
  in 1.21.4's jar, present in 1.21.5's), but removing it needs
  the clone block-tick tests reworked — they depend on strict
  placement (no own tick) to assert the copied ticks — so it
  stays until a clone-focused item takes it. `/setblock` and
  `/fill` had their strict removed in batch 16.

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
  Batch 15 (items 37 + 38 + 39 + 40). Item 37 (mobs/render.rs):
  PORTED. jeb_wool_rgb gains the entity-id offset (ticks/25 + id)
  — jar-verified in 1.21.4 SheepWoolLayer.render (floor(ageInTicks)
  /25 + state.id, mod 16, ARGB.lerp with (age%25 + frac)/25); the
  1.21.4 tree carried the 1.21.6 ColorLerper form without the id
  (1.21.5 moved the computation to SheepRenderState.getWoolColor
  with the same formula). The caller passes m.id and the tests are
  extended (id 1 starts at orange, id 16 wraps to white). NO-CHANGE:
  the mulPose Quaternionfc-vs-Quaternionf renames (AllayModel,
  CrossedArmsItemLayer) are API-only; the four setupRotations,
  SheepRenderer.extractRenderState and WitchItemLayer are
  bytecode-identical; the updateForLiving left-hand bool is
  internal to the Java item-model resolver (the Rust already
  distinguishes hands). Item 38 (game/maps.rs): NO-CHANGE, tree
  diff empty. CompassItem inventoryTick/useOn and EmptyMapItem.use
  are API-only (ServerLevel cast vs isClientSide, playSound
  overloads, local-slot reshuffles). The MapItemSavedData setDirty
  differences (checkBanners, tickCarriedBy, toggleBanner call
  setDirty in 1.21.5, not in 1.21.4) are server-save concerns with
  no Rust equivalent — the Rust models banners/frames as render
  data only. MapFrame entityId() vs getEntityId() is a
  record-accessor rename; ExplorationMapFunction's MapItem.create
  param type is API-only. Item 39 (game/dispenser.rs): PORTED.
  (1) shear_off_leashes removed — tryShearLivingEntity is
  byte-identical in both jars (Shearable + game event, no leash
  handling); the 1.21.6 leash-cutting arm does not exist in 1.21.4.
  (2) mob_can_use_slot rewritten to the 1.21.4 form (routed from
  batch 6): the LivingEntity base returns false, Mob allows every
  slot but BODY, Horse/Llama/Wolf allow them all; the
  Pig/AbstractHorse saddle arms (grown/tame) are 1.21.5 additions,
  and skeleton/zombie horses lose the body slot (no override —
  they cannot wear horse armour in 1.21.4). (3)
  mob_can_dispenser_equip rewritten: Pig/Strider have no override
  in 1.21.4 (just canPickUpLoot), AbstractHorse is (BODY && tamed)
  || loot — the saddle arms are 1.21.5, so a 1.21.4 dispenser
  cannot saddle a pig, strider or horse (Animal/AgeableMob/
  PathfinderMob add no override either). (4) Arrow
  #doPostHurtEffects in potions.rs: the named potion's effects at
  an eighth (integer division, min 1 tick), the custom effects
  UNSCALED — the Rust scaled every effect by
  potion_duration_scale, which does not exist in 1.21.4 (the
  lambda is i/8, verified). (5) The setGuaranteedDrop comment is
  carried (1.21.4: setDropChance(slot, 2.0F), verified equivalent
  via DropChances.withGuaranteedDrop). NO-CHANGE: MobBucketItem
  #spawn (createDefaultStackConfig param type), the two same-
  methods; the Player canUseSlot comment is fixed (every slot but
  BODY). Item 40 (game/frame_geometry.rs): NO-CHANGE, tree diff
  empty. Every SkyRenderer difference is render-pipeline API with
  identical geometry: buildEndSky/buildStars build GPU buffers in
  1.21.5 vs writing into a passed VertexConsumer in 1.21.4 (same
  vertices: sky disc at +/-100 with UV 16, 1500 stars from the
  same random draws); renderSkyDisc/renderStars use RenderPass/
  GpuBuffer vs VertexBuffer.drawWithRenderType; the sun/moon/
  sunrise diffs are mulPose signature renames. The Rust models
  frames/transforms only, with no buffer-API references. Lib check
  green (EXIT 0, 2m01s); --tests check green (EXIT 0, 3m16s).
  render.rs and dispenser.rs rustfmt-clean; potions.rs has only
  two pre-existing drift hunks (lines 260, 2004) outside the
  edited regions. Rank 2: 87/348.
  Batch 16 (items 41 + 42 + 43 + 44). Item 41 (game/raids.rs):
  NO-CHANGE, tree diff empty. bright_outside() is !hasFixedTime
  && skyDarken < 4 — exactly 1.21.4's isDay form (1.21.5's
  isBrightOutside is the same formula, a pure rename; the comment
  now says so). PatrolSpawner.tick and tick_village_siege return
  () — the 1.21.4 boolean/count returns are unmodelled and
  unobservable. The raider RaidId is saved directly as Int
  (raider.rs:713), the 1.21.4 form (1.21.5 looks it up via
  Raids.getId). PathfindToRaidGoal.recruitNearby, VillageSiege
  #trySpawn (snapTo vs moveTo, same), Raider.aiStep/die and
  pickUpItem are API-only or same-logic. Item 42
  (game/commands.rs): PORTED. (1) /setblock and /fill now notify
  neighbours via update_neighbors_at — 1.21.4's
  ServerLevel.blockUpdated, plain neighbour notification —
  replacing the 1.21.6 update_neighbours_on_block_set
  (affectNeighborsAfterRemoval exists in neither 1.21.4's nor
  1.21.5's Block class; 1.21.4's Level.setBlock with flags=2 runs
  neither blockUpdated nor the analog-output update, and the
  commands send the notice themselves). The commands.rs wrapper
  is replaced (it now reads the current state for the block);
  fill's updated list is plain positions, no old states. (2)
  strict REMOVED from /setblock and /fill: jar-verified a 1.21.5
  addition (0 occurrences of "strict" in 1.21.4's
  SetBlockCommand, FillCommand and CloneCommands; present in
  1.21.5's; 1.21.4's setBlock has no boolean param and
  FillCommand$Mode has only the filter field). block_input_state
  always normalizes (1.21.4's BlockInput.place does
  updateFromNeighbourShapes + overwriteWithDefinedProperties);
  the dispatcher trees drop the strict literal; 44 test command
  strings drop the token (all verified safe: fully-specified
  properties are restored by overwriteWithDefinedProperties, the
  rest have no neighbour-dependent properties, the fence sites
  assert nothing about connections) and the 2 note tests use
  explicit instrument=harp instead (one renamed). (3) The
  debug-world check REMOVED from /setblock, /fill and /clone
  (isDebug appears 0 times in 1.21.4's three command classes, 1
  time each in 1.21.5's). Clone keeps its strict (CloneCommands
  is outside item 42's classes, and the clone block-tick tests
  depend on its strict semantics — a routed gap, see below);
  place_state keeps its strict param for clone, and /place
  template's strict (structure settings) is untouched.
  NO-CHANGE: the teleportTo pitch clamp is already present
  (pitch.clamp(-90, 90) at commands.rs:1698, matching 1.21.4's
  Mth.clamp in teleportTo — routed from batch 5); /advancement
  is same-behaviour (a shared helper vs inline messaging);
  FillCommand.wrapWithMode is 1.21.5-only; the fill
  destroy-count (affected) matches 1.21.4's DESTROY filter; the
  block-entity clearing in place_state (remove without side
  effects on a block change) is 1.21.4's Clearable.tryClear.
  Item 43 (protocol/item/skip.rs): NO-CHANGE. All jar diffs are
  ByteBufCodecs anonymous-class numbering only ($24 vs $23, $28
  vs $26, $29 vs $27, $21 vs $20; fromCodecWithRegistries,
  idMapper, registry are same) — zero behaviour. The tree diff
  is just the component-id table, already re-pinned for 1.21.4
  by the mechanical layer. Item 44 (widely named:
  client/resources/model): NO-CHANGE. 15 vanilla classes, 85
  methods, 0 named in Horizon; module_status missing-both — the
  same shape as items 27 and 29. Lib check green (EXIT 0,
  2m32s); --tests check green (EXIT 0, 4m06s). vanilla.rs,
  entity_die_tests.rs and clone_block_ticks_tests.rs
  rustfmt-clean; data.rs had 3 drift hunks from the strict drops
  (fixed by hand to rustfmt's form) and terrain_cmds.rs 1 (fixed
  the same way); commands.rs, raids.rs and the other test files
  checked only (module roots / file-module declarers), with only
  pre-existing drift hunks elsewhere (creatures.rs 3893/6110/
  7088, persist.rs 1559, commands/extra.rs 127 — left alone).
  Rank 2: 91/348.
  Batch 17 (items 45 + 46 + 47 + 48). Item 45 (render/camera.rs):
  PORTED. (1) Camera distance is the flat 4.0 * scale (1.21.4's
  Camera.setup computes 4.0f * LivingEntity.getScale(), no
  camera_distance attribute, no mount consideration; the 1.21.5
  form carries). (2) depth_far is renderDistance * 4 with no
  cloud-range max (1.21.4's GameRenderer.getDepthFar; three call
  sites drop the cloud_range arg). (3) The spin is now 1.21.4's
  single spinning intensity: ScreenEffects keeps spinning /
  o_spinning / confusion_animation_tick (incremented per tick);
  tick merges the portal ramp (+0.0125), the Confusion ramp
  (+1/150) and the decay (-0.05), clamped 0..1, per
  LocalPlayer.handleConfusionTransitionEffect (read in full:
  portal step also closes containers and plays PORTAL_TRIGGER at
  intensity 0 — the nether_portal.rs gate now reads the pre-tick
  spinning intensity). spinning_matrix uses angle =
  (confusionAnimationTick + partial) * (7 if Confusion else 20)
  and intensity = lerp * scale^2. The Nausea blend fields,
  tick_spinning and nausea_blend_added are gone: BlendState
  exists in 1.21.4's MobEffectInstance and the UpdateMobEffect
  packet carries the flag-8 shouldBlend bit, but no 1.21.4
  renderer reads the blend factor (grep over LocalPlayer,
  GameRenderer, Gui, FogRenderer, LevelRenderer: 0 calls), so
  the remote.rs nausea branch is removed with it.
  (4) camera_overlays: the portal swirl shows when intensity > 0
  without Confusion, the confusion swirl when Confusion is
  present and screenEffectScale < 1 (alpha = intensity *
  (1 - scale)), per Gui.renderCameraOverlays. NO-CHANGE: the
  bobHurt/bobView mulPose renames, processBlurEffect (no
  PostChain in the Rust), render/renderLevel GPU API,
  takeAutoScreenshot and the tick null-check reorder are
  API-only; spinning_effect_matrix is identical in both jars.
  Tick granularity: confusionAnimationTick advances per game
  tick here, per frame in vanilla — the same approximation the
  1.21.5 form already made. Item 46 (game/mounts.rs): PORTED.
  The plain-carrot horse-diet arm is removed (neither 1.21.4's
  nor 1.21.5's AbstractHorse.handleEating mentions Items.CARROT
  — a 1.21.6 addition; the test asserts eating(carrot) == None)
  and the jump-meter comment now cites Gui.renderJumpMeter
  (1.21.4 has the private method, no JumpableVehicleBar class).
  NO-CHANGE: Strider mobInteract already checks
  stack.is(Items.SADDLE) (the 1.21.4 form, not 1.21.5's
  isEquippableInSlot); the strider finalize saddles + drop
  chance 2.0 (the equipSaddle outcome); Camel handleEating/tick
  and Strider floatStrider/spawnJockey are playSound overloads
  or renames. The stand-counter test (counter 20) is LEFT for
  item 205: the logic lives in equine.rs (stand_if_possible
  sets 20; 1.21.4's jar sets standCounter = 1) and changing the
  mounts.rs test alone would break it. Item 47
  (entity_nbt/hanging.rs): PORTED. Hanging entities save and
  load the tile position as TileX/TileY/TileZ ints (1.21.4's
  BlockAttachedEntity.addAdditionalSaveData /
  readAdditionalSaveData), not the 1.21.5 block_pos int array
  the Rust wrote. Load order: Tile keys, then block_pos (kept
  for saves this port wrote before the fix), then the summon
  position's block. Tests updated (Tile keys round-trip,
  block_pos absent from saves). NO-CHANGE: ItemFrame Facing as
  a 3D byte (default 0 = DOWN), ItemRotation plain getByte,
  ItemDropChance default 1.0 (read only when present),
  Invisible/Fixed plain booleans, the Item compound check — all
  already the 1.21.4 form; Painting variant-as-string + 2D
  facing byte match the VARIANT_CODEC encoding. The Cushion
  block_pos (entity_nbt.rs) is a 26.x entity, out of scope.
  Item 48 (game/scoreboard.rs): PORTED. TamableAnimal.getTeam
  is owner-first: a tamed pet answers its owner's team, else
  its own (1.21.4 checks getOwner().getTeam() first, then
  Animal.getTeam; the Rust had the 1.21.5 own-first order).
  getRootOwner is gone (a 1.21.5 addition; 1.21.4 uses the
  direct getOwner()): root_owner and root_owner_name are
  replaced by owner_name resolving the direct owner, and a pet
  owned by a pet now answers that pet's team. Tests updated
  (both-teams case answers the owner's; pet-owned-by-pet
  answers the pet's team). NO-CHANGE: Owner save/load is
  already the 1.21.4 putUUID form (IntArray + uuid_from_tag);
  the legacy username conversion (OldUsersConverter) is a
  shared gap in both trees; tame.rs's is_owner/owned_by already
  use the direct owner. Routed: the narrator rename
  (say_system_now -> say_now, say_system_queued -> say,
  say_system_chat_queued -> say_chat, and sayChat's missing
  empty check — all jar-verified 1.21.4 behaviour, GameNarrator
  has only sayChat/say/sayNow) spans 15 files with no owning
  item; it goes to item 299 (horizon-narrator crate) and the
  scoreboard.rs:885 call site stays say_system_now until then.
  Lib check green (EXIT 0, 1m18s, one warning fixed: the unused
  screen_effects import in remote.rs); --tests check green
  (EXIT 0, 2m08s). camera.rs, hud_overlays.rs,
  screen_effects.rs, kinds.rs, hanging.rs, nether_portal.rs,
  render_tick.rs, fov.rs and screen_backgrounds.rs
  rustfmt-clean (fov.rs and screen_backgrounds.rs needed one
  hand fix each in the edited regions); agent.rs, mounts.rs and
  scoreboard.rs clean; remote.rs, potions.rs, entity_nbt.rs and
  game.rs checked only, with only pre-existing drift hunks
  elsewhere (remote.rs 5549, potions.rs 260/2004 — left alone).
  Rank 2: 95/348.
  Batch 18 (items 49 + 50 + 51 + 52). Item 49
  (game/armor_stands.rs): PORTED. (1) The PlayerHead special
  model is removed: 1.21.4's data uses `head` + `kind: player`
  (items/player_head.json, identical in 1.21.5) and neither jar
  has a PlayerHeadSpecialRenderer (only SkullSpecialRenderer,
  byte-identical sizes) — the 1.21.5 tree's form carries. The
  variant is gone from horizon-assets special.rs (enum, parse
  and serialize arms); argument() reads the profile for
  Head{kind: player}; armor_stands.rs and mob_blocks.rs use
  Head for all kinds; item_render/special.rs renders a player
  head inside the Head arm (profile skin, translucent) and the
  extents arm is dropped (skull_layer("player") is already
  "player"). (2) The happy-ghast phrase is removed from the
  tick comment (batch 2's removal). NO-CHANGE:
  ArmorStandItem.useOn is API-only (createDefaultStackConfig
  LivingEntity-vs-Player param, snapTo/moveTo, playSound
  overload); swapWithEquipmentSlot already matches 1.21.4
  (equip_from_hand checks no canBeEquippedBy — the 1.21.5
  addition — and canUseSlot is vacuous for the four armour
  slots); equipOnInteract exists only in the protocol layer,
  correctly dropped on the 769 wire; the armor-stand save
  already uses HandItems/ArmorItems. Item 50 (widely named:
  world/level/block): NO-CHANGE. 26 vanilla classes, 77
  methods, 0 named in Horizon — the same shape as items 27, 29
  and 44. Item 51 (lan/data.rs): NO-CHANGE, comment-only.
  ListTag.addAndUnwrap / wrapIfNeeded are 1.21.5-only, but the
  behaviour (mixed-list {"": ...} wrappers) exists in 1.21.4
  inside NbtOps$HeterogenousListCollector (own wrapElement /
  isWrapper, javap-verified) and the Rust's nbt.rs already
  wraps/unwraps the same way, so only the lan/data.rs citations
  change. Item 52 (world/level/mod.rs): PORTED. (1) The dead
  update_neighbours_on_block_set is removed (23 lines): no
  callers since batch 16 removed the commands.rs call, the
  1.21.6 form, absent from the 1.21.5 tree. (2) Removal
  citations carried to the 1.21.4 trigger:
  BlockEntity.preRemoveSideEffects does not exist in 1.21.4
  (only setRemoved) — LevelChunk.setBlockState calls
  BlockState.onRemove, and LecternBlock.onRemove + popBook and
  JukeboxBlock.onRemove exist (all javap-verified). The
  removal.rs module doc, PreRemove doc, lectern_book_position
  doc (LecternBlock.popBook) and the level/mod.rs:796 doc now
  cite onRemove; the effects themselves (contents spill,
  jukebox pop, lectern book, dropItemStack math) are identical
  in both versions. The PreRemove / pre_remove identifiers
  stay (internal names). NO-CHANGE: BellBlock.onProjectileHit
  is a local-slot reshuffle (same logic; the Rust takes no
  entity param); the BlockEntity load/save codec-vs-MapCodec
  differences are the item-32 components form (already
  handled); the update.rs:348 piston preRemoveSideEffects
  comment belongs to that module's item. Lib check green
  (EXIT 0, 1m54s, no warnings); --tests check green (EXIT 0,
  2m06s). armor_stands.rs, special.rs (both), mob_blocks.rs,
  lan/data.rs and removal.rs rustfmt-clean
  (item_render/special.rs needed one hand fix in the edited
  region: the `if let` brace); level/mod.rs checked only
  (module root, no drift in the edited regions). Rank 2:
  99/348.
  Batch 19 (items 53 + 54 + 55 + 56). Item 53
  (render/structure_block.rs): PORTED. The load defaults are the
  1.21.4 plain-get forms: ignoreEntities and showboundingbox
  default to false (plain getBoolean, not 1.21.5's
  getBooleanOr(true)) and posY to 0 (plain getInt, not
  DEFAULT_POS (0,1,0)); the constructor defaults in new() stay
  (0,1,0)/true/true — those are the field initializers
  (javap-verified), and load overwrites them. The load tests now
  assert the 1.21.4 defaults. NO-CHANGE on the render box:
  BoundingBoxRenderable does not exist in 1.21.4 (a 1.21.5
  addition), but 1.21.4's StructureBlockRenderer gates match
  render_mode exactly (Save/Load only; the box iff Save or
  showBoundingBox; invisible blocks iff Save && showAir), and
  the mirror/rotation box math is the same source, so
  render_mode / renderable_box stand as the ported equivalents.
  strict stays — routed to item 394 (rank 3). Item 54
  (mobs/spawn_selection.rs): NO-CHANGE, comment-only. All diffs
  are API-only — WeightedList vs WeightedRandomList (rename),
  SpawnerData record accessors vs fields (same values), snapTo
  vs moveTo; SpawnerData#equals is 1.21.5-only. One citation
  updated (WeightedRandomList.getRandom). Item 55 (mobs.rs):
  PORTED. (1) A mob's fire_ticks starts at -1 (the Entity
  constructor sets -getFireImmuneTicks(), javap-verified), so
  the first lightning strike ignites a fresh mob. (2)
  extinguish_fire sets exactly 0 (clearFire), not min(0).
  (3) The 1.21.4 post-check in ai.rs tick: a non-burning alive
  entity not inside fire or lava leaves -1 after the pass —
  EVERY pass, not only when a fire went out in it. 1.21.4's
  applyEffectsFromBlocks has no wasOnFire gate (1.21.5 gates
  the reset on it); the flag predicate is the #fire tag or
  lava (the synthetic lambda, javap-verified). The wet-fire
  test expectations move 0 -> -1. (4) The dead leash_spin
  field is removed (write-only; the creaking.rs write is gone
  with it). (5) The travelFlying comments are fixed (no such
  method in either jar) and the LEAD_BREAK comment is carried
  (1.21.4 plays no sound for a snapped lead). NO-CHANGE: the
  Cow 0.4 volume was already correct (Cow.getSoundVolume is
  1.21.4-only); the RemotePlayer lerp math is exactly 1.21.4's
  lerpPositionAndRotationStep — InterpolationHandler does not
  exist in 1.21.4 at all, and the citations in remote.rs,
  entities.rs, fishing.rs, end.rs, interactions.rs and
  dragon.rs are a known naming lag, behaviour verified
  identical, deliberately not swept. KNOWN GAP: the
  entities.rs fire_ticks: 0 inits (6 sites) plus arrows.rs and
  fluids.rs stay 0, matching the 1.21.5 tree's scoping — the
  jar's -1 applies to all entities. Item 56
  (game/block_ticks.rs): PORTED. (1) The portal-ticket expiry
  gate is removed: 1.21.4's Ticket.timedOut is a pure age
  check (now - createdTick > timeout) and canTicketExpire is a
  1.21.6 addition — tick_portal is bare (the 1.21.5 tree form),
  portal_ticket_can_expire is deleted from game.rs and the
  levels.rs doc is fixed; the tick_portal doc now cites
  DistanceManager.purgeStaleTickets. (2) The tickChunks
  shuffle is added: 1.21.4's ServerChunkCache.tickChunks runs
  Util.shuffle(tickingChunks, level.random) before ticking
  (1.21.5 dropped it) — the Rust shuffles scratch.ticking with
  the level random in the explosion.rs pattern. Lib check
  green (EXIT 0, 17m12s, no warnings); --tests check green
  (EXIT 0, 53m32s, no warnings) — both slow because the disk
  quota forced deleting target/debug/incremental (3.6G,
  regenerable) mid-check and the cache rebuilt cold. One
  rustfmt incident: a bare `rustfmt <file>` piped to diff
  formatted game.rs and mobs.rs IN PLACE and recursed into
  19 submodule files; all unintended changes were reverted
  with git checkout and the two roots' edits re-applied, so
  the commit holds only the batch's 10 files. (Never pipe
  rustfmt's output for a module root — --check only.)
  creaking.rs, structure_block.rs, spawn_selection.rs,
  block_ticks.rs, ticket_levels.rs, levels.rs and
  wet_fire_tests.rs rustfmt-clean; ai.rs clean by --check;
  mobs.rs and game.rs checked only (module roots). Rank 2:
  103/348.
  Batch 20 (items 57 + 58 + 59 + 60). Item 57 (game/boats.rs):
  PORTED. (1) The chest-boat sneak hunk is carried (routed from
  batch 8): AbstractChestBoat.interact checks isSecondaryUseActive
  in both jars, byte-identical — a sneaking player opens the
  chest instead of the leash step, even with a lead in hand.
  (2) The bubble-column wobble's phase is the level's game time
  in 1.21.4 (10.0f * sin(0.5f * gameTime)), not the boat's
  tickCount (1.21.5) — tick_bubble_wobble takes game_time, the
  dead tick_count field is removed, both call sites (boats.rs
  and remote_vehicles.rs) pass world.game_time, and the tests
  advance the game time. NO-CHANGE: onAboveBubbleColumn is
  onAboveBubbleCol in 1.21.4 (a rename); the
  getGroundFriction / pushEntitiesUp move(Vec3i)-vs-move(DDD)
  difference is API-only; fallDistance float-vs-double is left
  at f64 (the 1.21.5 port's scoping); the tick lerp is the
  InterpolationHandler naming lag (item 55). Item 58
  (game/interaction_shapes.rs): NO-CHANGE, comment-only. The
  values are identical — 1.21.4 builds the cauldron/hopper
  shapes with literal Block.box calls (javap-verified: cauldron
  INSIDE box(2,4,2,14,16,14), hopper INSIDE plus the
  *_INTERACTION_SHAPE statics) and 1.21.5 with the column/boxZ
  helpers (1.21.5-only), so only the citations change. fallOn
  float-vs-double and the spawnDestroyParticles Player-vs-Entity
  overload are API-only. Item 59 (game/advancements.rs):
  PORTED. (1) The AdvancementToast fade is carried wholesale —
  the bytecode is identical in both jars: the label draws at
  full opacity until 1.5 s, then the title lines fade in with
  (fade * 252) | 0x40 (never below alpha 64); the 1.21.4 tree's
  1.21.6 form (label fades out, lines fade in from 0) is
  replaced, with advancement_title_alpha and its test added.
  (2) Toast sounds are un-deduped: 1.21.4 has no
  playedToastSounds set — every visibility change plays
  ui.toast.in/out (1.21.5 dedups each event once per update).
  NO-CHANGE: the showAdvancements toast gate was already
  removed (batch 12); the AdvancementRewards#grant playSound
  overload and ItemPredicate#test are API-only/same. Item 60
  (merchant.rs): NO-CHANGE, comment-only.
  DataComponentPredicate (1.21.4) and
  DataComponentExactPredicate (1.21.5) have identical subset
  semantics — both check Objects.equals for each expected
  component, extras allowed (javap-verified); only the getter
  interface differs, so only the citation changes.
  selectEnchantment's WeightedRandom form differs only in the
  weight-function argument (same selection); the Rust's
  enchant.rs select_with does its own weighted pick. Lib check
  green (EXIT 0, 7m32s, no warnings); --tests check green
  (EXIT 0, 4m35s, no warnings). boats.rs needed one hand fix
  in the edited region (the wobble assignment's line break);
  interaction_shapes.rs, advancements.rs and merchant.rs
  rustfmt-clean; remote_vehicles.rs has only pre-existing drift
  elsewhere (lines 282-293 — left alone). Rank 2: 107/348.
  Batch 21 (items 61 + 62 + 63 + 64). Item 61
  (game/native_block_entity_draw.rs): NO-CHANGE. The Rust's
  PoseStack is a plain Vec<Mat4> (push clones the top, pop
  guarded to len > 1) — behaviour-identical to both jars'
  push/pop; the List+lastIndex (1.21.5) vs Deque (1.21.4)
  internals are unobservable through it, and rotateAround's
  matrix+normal split equals Pose.rotateAround. ModelPart's
  mulPose(Quaternionfc)-vs-(Quaternionf) is API-only. Item 62
  (render/chunk/block_entity.rs): NO-CHANGE, comment-only. The
  jukebox load/save already is the 1.21.4 form — no
  isSameItemSameComponents same-record check (1.21.5-only), the
  clock is honoured only with a record plus the clock key
  (JukeboxSong.fromStack), the save writes the clock only when
  the song exists; RecordItem parses via ItemStack.parse when
  the key is present, else the item empties (and a playing song
  stops). The one edit: TrialSpawnerStateData ->
  TrialSpawnerData — the 1.21.4 jar has only TrialSpawnerData
  (the StateData name is 26.3-era). ROUTED: the same misnomer
  in mobs/trial_spawner.rs (items 88/140/245/268/315/339),
  game/trial_spawner.rs (item 245) and lan/chunks.rs (items
  163/252) belongs to those items. Item 63
  (horizon-command/snbt.rs): NO-CHANGE, comment-only.
  parseAsArgument / parseCompoundAsArgument / parseFully are
  1.21.5-only names, but the Rust's three functions implement
  1.21.4's shapes — parse_tag_argument is TagParser.readValue
  via NbtTagArgument.parse (one value at the cursor),
  parse_fully is TagParser.parseTag (one value, trailing data
  is an error). Citations updated to the 1.21.4 names. Item 64
  (game/natural_spawning.rs): NO-CHANGE.
  close_enough_for_spawning is ChunkMap.
  euclideanDistanceSquared < 16384 with the chunk-centre offset
  in both (the Vec3-field vs Entity.getX/getZ difference is
  API-only); the TriState-vs-boolean hasPlayersNearby is
  unmodelled — the Rust computes closeness directly from player
  positions, which is what 1.21.5's TriState.DEFAULT falls back
  to; collectSpawningChunks and forEachBlockTickingChunk
  (1.21.5-only) have no Rust counterpart to change. No
  citations to fix. --tests check green (EXIT 0, 16m08s, no
  warnings) for horizon-client-render + horizon-command.
  block_entity.rs and snbt.rs rustfmt-clean. Rank 2: 111/348.
  Batch 22 (items 65 + 66 + 67 + 68). Item 65 (widely named:
  client/renderer/block/model): NO-CHANGE. 10 vanilla classes,
  62 methods, 0 named in Horizon — the same shape as items 27,
  29, 44 and 50. Item 66 (game/tnt.rs): PORTED — the
  cross-cutting `tnt_explodes` removal (routed from batch 9).
  RULE_TNT_EXPLODES does not exist in 1.21.4 (added in 1.21.5;
  jar-verified: 1.21.4's GameRules has only
  RULE_TNT_EXPLOSION_DROP_DECAY). Removed across 14 files: the
  gamerules.rs registration, the save.rs default (57 -> 56),
  the dead fixes_c.rs rename mapping (tntExplodes can never
  appear in a pre-rename save), the tnt_prime_allowed gate,
  the PrimedTnt tick's despawn-without-blast, the useItemOn
  disabled-message gate, the arrow scene field plus the TNT
  prime gate, the explosion wasExploded gate, the three
  dispenser gates, the three sulfur_cubes gates, the
  sulfur_cube thread-local rule sharing (set_tnt_explodes
  deleted; prime/tickFuse always behave as on; the two
  rule-off tests removed, the remaining fuse test's module
  renamed fuse_tests), and the chat.rs/game.rs rule-sharing
  calls. 1.21.4's TntBlock calls explode() directly (prime()
  is 1.21.5-only) and playerWillDestroy checks isCreative()
  (not getAbilities().instabuild). One warning fixed on the
  way: the Text import in sulfur_cubes.rs went unused with the
  disabled-message block. Item 67 (game/beacon.rs): PORTED.
  BeaconRenderer.getViewDistance is a flat 256 blocks in
  1.21.4 (1.21.5 culls at the effective render distance times
  16) — beacon_geometry's cull is now the constant 256.0.
  NO-CHANGE: the beacon decode reads no CustomName in either
  tree (a shared gap); the beam sections are (colour, height)
  tuples in both (the BeaconBeamOwner$Section vs BeaconBeamSection
  class move is API-only); shouldRender's BlockEntity-vs-
  BeaconBlockEntity getBlockPos is API-only. Item 68
  (world/level/hooks.rs): NO-CHANGE. ButtonBlock#getShape is a
  Function table in 1.21.5 vs the explicit face/facing/powered
  switch in 1.21.4 with identical shapes (data-driven in the
  Rust, no code to change); ButtonBlock#entityInside is same;
  SpongeBlock#tryAbsorbWater is a playSound Player-vs-Entity
  overload; PistonHeadBlock#playerWillDestroy's
  preventsBlockDrops() is exactly getAbilities().instabuild in
  1.21.5 (javap-verified), and the concept is unmodelled in
  both trees. Lib check green (EXIT 0, 2m12s) after a runtime
  restart lost the first run's result (empty log; re-ran from
  scratch); --tests check green (EXIT 0, 7m09s, no warnings).
  gamerules.rs, fixes_c.rs, tnt.rs, arrow_probe.rs,
  arrows_client_tests.rs, explosion.rs, dispenser.rs,
  sulfur_cubes.rs, chat.rs and beacon.rs rustfmt-clean;
  arrows.rs and sulfur_cube.rs needed one hand fix each in the
  edited regions (a match-arm collapse, a stray blank line);
  save.rs, creatures.rs and game.rs checked only (module
  roots / pre-existing drift elsewhere — left alone). Rank 2:
  115/348.
  Batch 23 (items 69 + 70 + 71 + 72). Item 69
  (assets/item_model.rs): NO-CHANGE. The tree diff is only the
  DATA_COMPONENT_TYPES table, already the 1.21.4 list (67
  entries, with hide_tooltip / hide_additional_tooltip and
  map_color). GrassColor#get's inline math matches the Rust's
  biome.rs sample (clamped temperature, downfall x temperature,
  (y<<8|x) index, bounds-checked fallback);
  ItemStackRenderState.isLeftHand is write-only in 1.21.4
  (nothing reads it), so the Rust correctly carries no such
  field (the display context already has the left-hand
  variants). Item 70 (world/level/bonemeal.rs): PORTED.
  GrassBlock.performBonemeal takes the biome's FIRST flower
  feature in 1.21.4 (flowers.get(0)) and a random one in
  1.21.5 (flowers.get(random.nextInt(size))) —
  grass_bonemeal_effect now takes list[0]. NO-CHANGE: the
  spreadable-neighbour helper mirrors 1.21.5's
  getSpreadableNeighbourPos extraction, but its callers are
  dead in 1.21.4 (no BushBlock.performBonemeal, no FireflyBush)
  so the shuffle flag is unobservable here. KNOWN SHARED GAP:
  the Rust's flower list is built from the
  can_spawn_from_bone_meal tag, not the biome's flower-features
  list (both trees). Item 71 (data_cmd.rs): NO-CHANGE,
  comment-only. NumericTag#doubleValue/intValue/shortValue are
  1.21.5-only names for getAsDouble/getAsInt/getAsShort
  (identical behaviour); the Rust's numeric() cites getAsDouble
  now. Item 72 (game/allays.rs): NO-CHANGE. Allay.travel's
  1.21.4 isControlledByLocalInstance gate is a client-side/no-AI
  guard — the server tick only travels AI-enabled mobs, so it
  is already implied (comment recorded on allay::travel);
  duplicateAllay snapTo/moveTo is a rename; mobInteract's
  playSound overloads are API-only;
  updateDuplicationCooldown's unconditional decrement matches
  1.21.4 (the 1.21.5 server-gate + setDuplicationCooldown form
  differs), and the DATA_CAN_DUPLICATE sync has no Rust
  counterpart in either tree (a shared gap). --tests check
  green (EXIT 0, 4m10s, no warnings) for horizon-client +
  horizon-world. bonemeal.rs, data_cmd.rs and allay.rs
  rustfmt-clean. Rank 2: 119/348.
  Batch 24 (items 73 + 74 + 75 + 76). Item 73 (widely named:
  client/renderer): NO-CHANGE. 7 vanilla classes, 53 methods, 0
  named in Horizon — the same shape as items 27, 29, 44, 50
  and 65. Item 74 (world/level/sculk.rs): NO-CHANGE. All four
  diffs are Level.playSound Player-vs-Entity overloads
  (SculkSensorBlock#activate/tick, SculkSpreader$ChargeCursor
  #update, SculkVeinBlock#attemptPlaceSculk) — API-only, same
  sounds. Item 75 (game/cat_spawner.rs): NO-CHANGE.
  Cat.finalizeSpawn's moon/tag pick already matches 1.21.4 —
  tame::cat_variant draws uniformly over 10 variants (11 on a
  full moon), which is exactly Registry.getRandomElementOf over
  the verified tag data (default_spawns = the first 10 in
  registry order; full_moon_spawns = those 10 + all_black); the
  swamp-hut ALL_BLACK + persistence rule is already applied on
  the spawner path (spawn_cat checks cats_spawn_as_black). The
  CatSpawner#tick int-vs-void return is unobservable (the count
  is ignored). ROUTED GAP (item 188, spawning.rs): an
  egg/summon-spawned cat inside a hut does not get ALL_BLACK —
  finalize has no world handle for the structure check. Item 76
  (game/persist.rs): PORTED. (1) PistonMovingBlockEntity.
  saveAdditional writes facing as an INT in 1.21.4
  (get3DDataValue + putInt); 1.21.5 stores a byte via
  Direction.LEGACY_ID_CODEC (a BYTE codec, javap-verified) —
  the Rust wrote a byte; now Tag::Int. The load already reads
  it via number() and maps through Direction::ALL (Down/Up/
  North/South/West/East = from3DDataValue). (2) The piston tick
  places the moved block with flags 84 in 1.21.4, not 340 —
  340 adds 256 SKIP_BLOCK_ENTITY_SIDEEFFECTS, a 1.21.5-only
  flag. NO-CHANGE: the blockState NbtUtils form,
  progress/extending/source plain gets (same defaults), and
  preRemoveSideEffects absent in 1.21.4 (already handled in
  batch 18, item 52). --tests check green (EXIT 0, 6m06s, no
  warnings) for horizon-client + horizon-world. pistons.rs
  rustfmt-clean; persist.rs checked only (module root,
  pre-existing drift elsewhere — left alone). Rank 2: 123/348.
  Batch 25 verification (2026-11-26, restart after the agent stopped
  on a context limit at c4295402). The WIP touched only the item-80
  gamerule plumbing and one item-79 comment; it closed no item and
  updated neither work-items.json nor rank2-progress.json, so the
  committed counts stand (rank 2: 123/348). VERIFIED against the
  1.21.4 named server jar: (1) RULE_DOFIRETICK exists in 1.21.4
  GameRules and no FIRE_SPREAD/ALLOWFIRE rule does;
  LavaFluid.randomTick and FireBlock.tick both read RULE_DOFIRETICK
  only, while 1.21.5's LavaFluid.randomTick reads RULE_DOFIRETICK
  and RULE_ALLOWFIRETICKAWAYFROMPLAYERS — so the WIP's replacement
  of the fire_spread_radius_around_player rule with doFireTick and
  of can_spread_fire_around with the bare do_fire_tick read is
  correct; `canSpreadFireAround` is a Horizon name, it occurs in
  neither jar. (2) 1.21.4 CampfireBlock has the VIRTUAL_FENCE_POST
  field and 1.21.5 the SHAPE_VIRTUAL_POST field, so the beehive.rs
  comment correction is correct. REMAINING in batch 25: item 77
  (gfx.rs WorldBorderRenderer.render), item 78
  (natural_spawn_fixture.rs Slime/Zombie), item 79 (beehive.rs:
  CampfireBlock.entityInside, placeLiquid, Occupant.of,
  IGNORED_BEE_TAGS), item 80 (fluids.rs: BarrierBlock
  canPlaceLiquid/pickupBlock, FlowingFluid.tick lava gate).
  Rank 2: 123/348.
  Batch 25 (2026-11-26, restart continued): items 77, 78, 79, 80, all
  PORTED. Item 77 (gfx.rs WorldBorderRenderer.render): the Rust was the
  1.21.5 cached-buffer form (BorderRenderer with needs_rebuild/last_border,
  rebuildWorldBorderBuffer's 16 vertices, closestBorder nearest-first draw
  order). 1.21.4's WorldBorderRenderer has only render(WorldBorder, Vec3,
  double, double) (javap vs the 1.21.4 client jar): it emits the wall as
  one-block segments in a fixed east, west, south, north order, with no
  closestBorder (that method does not exist on 1.21.4's WorldBorder) and no
  0.01 inset (no RENDERING_OFFSET in either 1.21.4 or 1.21.5). Fixed: sides()
  now returns the fixed [east 3, west 1, south 0, north 2] order filtered by
  proximity; closest_border + java_double_compare removed; the 0.01 OFFSET
  inset removed (the wall now sits exactly on the border bounds); the
  far-outside early return removed (1.21.4 has only the deep-inside return;
  nothing is drawn far outside either way). Kept the port's cached
  single-quad buffer and texture-matrix scroll offset: the same picture as
  1.21.4's per-frame segmented emission, since the texture wraps and u is
  linear in the wall. Docs corrected (prepareDynamicTransforms is not a
  vanilla method in either version). Item 78 (natural_spawn_fixture.rs
  Slime/Zombie): PORTED — Zombie.canHoldItem in 1.21.4 checks Items.EGG (the
  single item); 1.21.5 checks ItemTags.EGGS. The #minecraft:eggs tag does not
  exist in 1.21.4 (no eggs.json in the jar's data, no EGGS field in 1.21.4's
  ItemTags class — javap-verified; both exist in 1.21.5), so the Rust's
  item_in_tag("minecraft:eggs") was always false: a baby zombie riding a
  chicken would wrongly pick up an egg. mob_looting.rs now checks item ==
  "minecraft:egg" (the binding is &*stack.item, as stack.item is Arc<str>).
  NO-CHANGE: Slime.readAdditionalSaveData (getInt/getBoolean vs
  getIntOr/getBooleanOr with the same 0/false defaults); Zombie.finalizeSpawn
  (Chicken.moveTo in 1.21.4 vs snapTo in 1.21.5 — the Rust's create() sets
  position + previous + yaw the same way, only the snapTo comment corrected
  to moveTo; the Halloween-head drop chance is a direct
  armorDropChances[HEAD] = 0.0 store in 1.21.4, which the Rust's
  set_drop_chance already does); Zombie.readAdditionalSaveData (identical
  defaults; the DrownedConversionTime arm matches on all producible inputs).
  Item 79 (beehive.rs): PORTED — IGNORED_BEE_TAGS is now the exact 1.21.4
  30-entry set from BeehiveBlockEntity's <clinit> (javap-verified, in order);
  the Rust had 25 (the 1.21.5 set): removed drop_chances, equipment,
  fall_distance, sleeping_pos; added ArmorDropChances, ArmorItems,
  FallDistance, HandDropChances, HandItems, HurtByTimestamp, SleepingX,
  SleepingY, SleepingZ. NO-CHANGE: Occupant.of's nectar read,
  CampfireBlock.entityInside, placeLiquid. Item 80 (fluids.rs): PORTED — the
  LevelAccess trait default can_spread_fire_around returned true with the
  stale "(fire-spread radius around players)" comment (a 26.x concept); it
  now returns self.game_rules().do_fire_tick, matching 1.21.4, where
  LavaFluid.randomTick, FireBlock.tick and lightning fire all spread only
  while doFireTick is on (javap-verified; the GameLevel override already did
  this). Default rules have do_fire_tick=true, so fixtures keep returning
  true. NO-CHANGE: BarrierBlock canPlaceLiquid/pickupBlock, FlowingFluid.tick
  (value PartialEq != matches 1.21.4's .equals()). cargo check green for
  horizon-client-render + horizon-world (5m15s, no warnings) and
  horizon-client (1m04s, no warnings). --tests: horizon-client-render lib
  2053 passed / 4 failed, horizon-world lib 444 passed / 3 failed,
  horizon-client lib 1263 passed / 4 failed — all 11 failures proven
  pre-existing by re-running them with the batch stashed (still failing).
  horizon-client-lib lib test target does not compile on this tree: 6 errors
  in block_entity_item.rs tests (spawn_egg_on_spawner called with 2
  arguments, defined with 1) — also proven pre-existing the same way;
  test-health item, CI crate-tests will show it. The border test
  near_wall_is_drawn_and_tinted passes with the new fixed order. Rank 2:
  127/348.
  Batch 26 (2026-11-26, restart continued): items 81, 82, 83, 84.
  Item 81 (horizon-command/src/item.rs): NO-CHANGE. 1.21.4's
  ItemPredicateArgument has its own parse/listSuggestions over its
  grammarWithContext field (grammar.parseForCommands + Util.allOf +
  Result::test; suggestions via grammar.parseForSuggestions); 1.21.5's
  extends ParserBasedArgument and inherits the same two calls, so the
  entry points are observably identical. ComponentPredicateParser.
  createGrammar has the same 10 lambdas and rule structure in both
  versions; the ResourceLookupRule constructor and
  lambda$createGrammar$9's Scope parameter are internal packrat
  plumbing. Decode is the Tag form in 1.21.4 (ComponentWrapper/
  PredicateWrapper decode(reader, RegistryOps<Tag>, Tag); Context
  passes a Tag) and the Rust's decode_component/decode_predicate take
  &Tag — the 1.21.4 form; 1.21.5's Dynamic form validates the same
  values through the same codec. ItemParser holds TagParser<Tag> +
  RegistryOps<Tag> in 1.21.4 (passed as parameters in 1.21.5); the
  Rust's ItemState parses SNBT to a Tag and validates with
  decode_component — same unknown-item/component/repeated/malformed
  errors. The pseudo maps hold only count in both versions (javap
  <clinit>), which the Rust handles. Item 82 (level/tick.rs): PORTED —
  FireBlock.tick writes the aged fire with flags 4 in 1.21.4 (javap
  offset 179-180, iconst_4) and 260 in 1.21.5; the Rust had
  set_block(pos, state, 260) at tick.rs:652, now
  super::flags::UPDATE_INVISIBLE (260 is UPDATE_NONE = 256 + 4, and
  256 is the 1.21.5-only UPDATE_SKIP_BLOCK_ENTITY_SIDEEFFECTS bit).
  The prime_tnt doc now names 1.21.4's TntBlock.explode (1.21.5
  renamed it prime and gated it on the tntExplodes rule, which 1.21.4
  does not have); behaviour already matched. NO-CHANGE: bootStrap's
  seven 1.21.5-only flammable blocks (SHORT_DRY_GRASS, TALL_DRY_GRASS,
  WILDFLOWERS, LEAF_LITTER, CACTUS_FLOWER, FIREFLY_BUSH, BUSH) do not
  exist in the 1.21.4 jar, and the Rust's fire odds come from the
  per-version block-behaviours.json; the tick gate (doFireTick via
  can_spread_fire_around after item 80) and the rest of fire_tick
  already match 1.21.4. Item 83 (game/dragon.rs): PORTED (comment
  only) — breath_affect_mob cited setPotionDurationScale(0.25) for the
  flame phase and DragonFireball.onHit; that method does not exist in
  1.21.4 (no such method on 1.21.4's AreaEffectCloud; 1.21.5 added it
  with DEFAULT_POTION_DURATION_SCALE). 1.21.4's quarter is
  AreaEffectCloud.serverTick's mapDuration(duration / 4); the comment
  now says so. Behaviour verified matching throughout: EndCrystal
  read/add save data (beam_target as an IntArray of 3 via NbtUtils,
  ShowBottom default true, write IntArray + ShowBottom); hurtServer
  identical (ldc/ldc_w noise only); EnderDragon.aiStep is the 1.21.4
  lerpSteps form (tick_dragon + tick_remote_dragon step 1/nth toward
  the sent position/yaw), not 1.21.5's InterpolationHandler;
  onCrystalDestroyed identical in both versions (Player-or-nearest,
  10.0 explosion on the head when nearest, phase strafe callback);
  readAdditionalSaveData matches on producible inputs (Phase::from_id
  clamps like getById; DragonDeathTime contains-guarded);
  sanitizeScale identical (returns 1.0);
  DragonSittingFlamingPhase.doServerTick differs only in the
  1.21.5-only setPotionDurationScale call and the HARM->INSTANT_DAMAGE
  rename — the scale only affects non-instant effect durations and
  both clouds carry only instant damage, and the Rust's instant
  magnitudes (3.0 flame / 6.0 fireball at the cloud's 0.5 scale) match
  1.21.4 exactly (cloud passes 0.5 to applyInstantenousEffect,
  javap-verified); DragonStrafePlayerPhase.doServerTick identical;
  EnderDragonRenderer/DragonFireballRenderer render methods identical
  (constant-pool noise only; extractAdditionalHitboxes is 1.21.5-only).
  Noted edge (no fix): a non-numeric DragonPhase tag coerces to
  0/Holding in 1.21.4 (contains + getInt) while the Rust keeps Hover —
  not producible by vanilla saves. Item 84 (hud.rs): NO-CHANGE.
  PackSelectionScreen.loadPackIcon is identical except the
  DynamicTexture constructor (API-only). The Rust's icon_texture
  reproduces the naming exactly: Util.sanitizeName lowercases
  (Locale.ROOT) then replaces chars failing
  ResourceLocation.validPathChar ([a-z0-9_-./]) with '_', and the sha1
  is Hashing.sha1().hashUnencodedChars(id) (UTF-16LE) — both
  javap-verified. TextureAtlas.dumpContents writes name.png + name.txt
  in both versions; the 1.21.5 change is the writeAsPNG overload (gl id
  vs GpuTexture + identity operator) — API-only; dumpSpriteNames sorts
  by key and writes '%s\tx=%d\ty=%d\tw=%d\th=%d' with getX/getY/
  contents width/height, which the Rust's dump_atlas matches.
  TextureAtlas upload/cycleAnimationFrames/getHeight/constructor differ
  only in the GL-id vs GpuTexture backend (1.21.5 adds a null-texture
  guard and passes the texture to tickAndUpload/uploadFirstFrame); the
  Rust's wgpu atlas + SpriteAnimator reproduce the observable
  behaviour. cargo check green for horizon-world + horizon-client
  (1m02s, no warnings). --tests: horizon-world lib 444 passed / 3
  failed, horizon-client lib 1263 passed / 4 failed — the same 7
  pre-existing failures as batch 25 (datafix x3, lan_particles,
  lan_screen, launch_api, spill). NOTE: the horizon-client lib run was
  interrupted by a runtime restart mid-run (session proc_897ba7b003e7,
  empty recorded log); the prebuilt test binary it had produced was
  re-run directly and completed in 13s with the baseline result.
  Rank 2: 131/348.
  Batch 27 (2026-11-26, restart continued): items 85, 86, 87, 88.
  Item 85 (game/mob_blocks.rs): NO-CHANGE.
  LivingEntityRenderer.setupRotations is 135 lines in both versions
  (only constant-pool indices differ) and
  CustomHeadLayer.translateToHead is 32 vs 32 with an empty diff.
  extractRenderState differs only in one call: 1.21.4's
  ItemModelResolver.updateForLiving(headItem, headStack,
  ItemDisplayContext.HEAD, false, entity) passes a boolean the
  1.21.5 4-arg form drops — stored into
  ItemStackRenderState.isLeftHand; a head item is never in a hand, so
  the picture is identical. extractAdditionalHitboxes is 1.21.5-only.
  Noted pre-existing gap (no fix — identical vs 1.21.5, not a version
  delta): a player named Dinnerbone/Grumm should flip upside-down
  when their cape is shown (isEntityUpsideDown reads getName() and
  requires isModelPartShown(CAPE)), but the Rust flips only on
  Mob.custom_name, which comes solely from DATA_CUSTOM_NAME/NBT and is
  never set from a player's username, so players with those usernames
  never flip. The cape bit is tracked (hidden_model_parts bit 0) but
  not consulted by the flip; fixing needs username plumbing into the
  flip path — a backlog item, not a port delta. Item 86
  (game/stats.rs): NO-CHANGE. ServerStatsCounter: 1.21.4 has
  fromJson(JsonObject), getKey(Stat), getStat(StatType, String) plus
  parseLocal/toJson lambdas; 1.21.5 has createTypedStatsCodec +
  lambdas instead. Verified by javap: 1.21.4's toJson hand-builds
  {"stats": {type-id: {stat-id: int}}, "DataVersion": <current>}; the
  Rust's to_json writes exactly that shape, and DATA_VERSION = 4189 is
  1.21.4's data version. PhantomSpawner.tick: 1.21.4 returns int (spawn
  count), 1.21.5 returns void; ServerLevel.tickCustomSpawners
  pops/discards the return in 1.21.4, so observably identical. The
  Rust's spawn_phantoms matches; gamerule ids are correctly mapped by
  the datafix layer (("doInsomnia", "minecraft:spawn_phantoms"),
  ("doMobSpawning", "minecraft:spawn_mobs")). Item 87 (mobs/undead.rs):
  PORTED — ZombifiedPiglin.customServerAiStep: 1.21.4 (after
  maybeAlertOthers) does `if (isAngry()) this.lastHurtByPlayerTime =
  this.tickCount;` — 1.21.5 has no such block. An angry piglin counts
  as recently-hurt-by-a-player for loot/XP for as long as it stays
  angry. Fix in zombified_piglin_step (after the anger if/else so a
  piglin whose anger just hit 0 doesn't refresh): `if m.extra.anger >
  0 { m.ai.last_hurt_by_player = 100; }`. Caveats: the Rust's
  single-countdown model can't express vanilla's tick-stamp tail —
  after anger ends the Rust's window is 100 ticks, vanilla's is
  ~age-in-ticks; an angry-at-a-non-player death credits the local
  player for statistics in the Rust (vanilla credits nobody, identity
  null) while loot/XP match. The setTarget companion (1.21.4-only `if
  (target instanceof Player) setLastHurtByPlayer(player)`) is subsumed
  in practice: a new player target starts anger on the next AI step
  and the refresh then covers loot/XP/credit; only uncovered window is
  ≤1 tick between target-set and the next step, plus the
  target-cleared-before-the-step case. MoveThroughVillageGoal.canUse
  (191 vs 191, empty diff) and Drowned.okTarget (17 vs 17, empty diff)
  identical; Drowned.travel's 1.21.4-only
  isControlledByLocalInstance() guard is vacuous (all simulated mobs
  are locally controlled). Two new tests (piglin_anger_tests,
  horizon-client-lib): an angry piglin counts as hurt by a player
  (timer 100 after one step), a calm piglin keeps no player-kill
  memory — both pass. Item 88 (widely named: world/entity): PORTED —
  InsideBlockEffectApplier/StepBasedCollector is 1.21.5-only: the
  Rust's arrows.rs Collector deduped effects per step and applied them
  in fixed type order, but 1.21.4's Entity.applyEffectsFromBlocks calls
  BlockState.entityInside + entity.onInsideBlock immediately per block
  in sweep order, with no collector. The divergent cases are exactly
  water before an igniter (water-then-fire: 1.21.4 leaves the arrow
  burning at 160, collector clears to 0; water-then-lava likewise 300
  vs 0); fire/lava ignites commute (both max). Fix:
  apply_block_effects now passes &mut fire_ticks into
  check_inside_blocks, which applies fire/lava/water inline per block
  in sweep order; the Effect enum, EFFECT_ORDER and Collector were
  deleted. Two new tests (arrows::client_tests, horizon-client-lib,
  server path, 1 tick): water-then-fire leaves the arrow burning
  (fire_ticks 159) and water-then-lava (299) — both pass. The rest of
  the item is NO-CHANGE by hand-checked normalized bytecode diffs:
  AgeableMob (explicit 0-initialization / getIntOr default 0),
  ConversionType$1.convert (same sequence; only the
  lastHurtByPlayerTime field rename), EntityAttachments.scale/
  Builder.build (refactor only), EntitySelector lambdas (identical),
  EquipmentSlot$Type/Group (1.21.5 adds SADDLE type + group; the Rust
  has neither, matching 1.21.4), Leashable/LeashData (same NBT format;
  the 1.21.5 CODEC is 1.21.5-only), Marker (1.21.4 keeps a data
  compound; the Rust preserves it via persist.raw),
  SpawnPlacements.checkSpawnRules (same), DropChances/EntityEquipment
  (1.21.5-only container move; same constants/behaviour),
  EntityReference (1.21.5-only; the Rust resolves owners by UUID, the
  1.21.4 form). Mob$1 is a switch-map class in both jars (the anonymous
  Container with getTheItem/setTheItem is AbstractHorse$1, the
  saddle-slot ContainerSingleItem built in the AbstractHorse
  constructor, removed in 1.21.5 when the saddle became a real
  equipment slot; the Rust's HorseInventoryMenu already models the
  saddle slot). EntityEvent: shared events have identical values;
  1.21.4 has ATTACK_BLOCKED=29 and SHIELD_DISABLED=30 (1.21.5 has
  DROWN_PARTICLES=67, SADDLE_BREAK=68 instead). The local-player shield
  sounds are already correct; the gap is the server-to-client broadcast
  + remote handling of 29/30 (never broadcast, remote handler lacks
  them) — a LAN-only gap, recorded, no fix in this batch (needs
  defender entity-id plumbing plus care to avoid double-play).
  DROWN_EVENT=67 handler is dead code in 1.21.4 (no 1.21.4 server ever
  sends it) — left as a cleanup candidate. cargo check green for
  horizon-client (1m39s). --tests: piglin 2/2, arrow 2/2 (both
  horizon-client-lib). Rank 2: 135/348.
  Test-health (2026-11-26, alongside batch 27): crate-tests FAILED on
  runs 45818664429 and 45823137761 with the same 11 failures (protocol
  item hash, world datafix x3, render packs x3, client lan_particles,
  lan_screen, launch_api, spill). All 11 fixed and verified locally:
  protocol skip.rs gains the synthetic mob-variant ids 72-88/90-95 as
  a varint (1.21.4 has 67 component types; PAINTING_VARIANT=89 is a
  1.21.5 id); block_entity_item.rs spawn-egg test re-scoped to the
  1.21.4 type-only form (neither version has Spawner.setEntityData,
  and SpawnEggItem.useOn calls only setEntityId); datafix fix_display
  parses 1.20 display.Name/Lore JSON into components instead of copying
  raw strings; the three datafix tests re-scoped to 1.21.4 (level data
  keeps Difficulty/GameRules/Spawn, chunk palette keeps Name, both
  fixes past 4189); the three pack tests re-scoped to formats 46/61
  (1.21.4 has no min/max_format or minors); launch-reference.md
  regenerated (61/46.0/1.21.4); the vibration test's particle id 58
  corrected to 46 (1.21.4's ParticleTypes order); remote GameRules
  button follows gamemaster (the 1.21.5 tree has the same test/implementation
  mismatch — a pre-existing inconsistency in the parent, not a delta).
  The spill test's cause was a real data-loss bug, not a test issue:
  canonical() dropped a placed block entity's components (it only
  dropped the `components` key, never merging them as 1.21.4's
  saveWithoutMetadata does), so a placed ominous banner lost its
  item_name/rarity/tooltip_display on save and on load. Fixed by
  merging the components into the tag as top-level `minecraft:*` keys
  in canonical(), reading them back through a new
  block_entity_components() helper in decode_banner and the copper
  golem statue branch, and using the same helper in
  collected_components() (pick-block). Local verification after the
  fixes: horizon-protocol 423 passed / 2 failed (both network tests,
  proven pre-existing by the stash test — the VM proxy intercepts one,
  EPERM on a UDP bind the other; the item-hash test passes),
  horizon-world 447/0, horizon-client-render 2055/2 (overlays fixed
  after the run; the other failure is the local HDR/environment one,
  still being identified), horizon-client 1267/0. Fixing the
  block_entity_item compile error also made horizon-client-lib compile
  for the first time on this tree: 2407 passed / 24 failed. Ten of the
  24 were 1.21.4-only and are fixed here: the world-gen settings
  cluster (7) — the writer stores settings in
  data/minecraft/world_gen_settings.dat (the 1.21.5 form) but
  current_world_gen_settings() only read level.dat, so
  current_world_gen_settings() + generators() now fall back to the
  file (level.dat wins when it carries the settings); the TNT owner
  test (1.21.4's PrimedTnt.addAdditionalSaveData writes only
  block_state/explosion_power/fuse, javap-verified — the test asserted
  an owner the 1.21.5 form writes); the hub self-pose test
  (first_tick hand-wrote a trailing showAdvancements bool, a 1.21.5
  addition — removed; flushDirty already omitted it); the server-packs
  header test (asserted 97.1, the client correctly sends 46.0).
  After those, horizon-client-lib is 2417/14 — exactly the 14 that
  fail identically on the 1.21.5 parent tree (item_sync x2, lan
  advancements x5, projectiles, realms, save chunks x4, title
  optimization), pre-existing programme test-health debt, not
  1.21.4 regressions.
  Batch 28 (2026-11-26, partial: items 89, 90, 91; item 92 held on the
  parent's CI-green priority steer — no more rank-2 items opened until
  CI is green). Item 89 (horizon-assets/item_bake.rs): PORTED.
  QuadCollection is 1.21.5-only; 1.21.4's ItemRenderer.renderModelLists
  draws each culled direction in Direction order (down, up, north,
  south, west, east), then the unculled quads, while the Rust ordered
  unculled first then north/south/east/west/up/down (the
  QuadCollection.getAll order). Fix in bake_resolved: culled quads in
  Direction::ALL order, then unculled; comments now cite
  renderModelLists. The order is the draw order (item_render.rs emits
  baked.quads in list order), so translucent quads blended in the wrong
  order. ItemTransform.apply matches 1.21.4 exactly (left-hand
  negation, translate-rotate-scale then -0.5 in LayerRenderState.render);
  ItemTransform$Deserializer defaults/clamps match (translation x0.0625
  clamp +/-5, scale clamp +/-4). New test quads_follow_the_client_draw_order:
  a cube's culls read [Down, Up, North, South, West, East, None] —
  horizon-assets item_bake 3/3 pass. Item 90
  (horizon-client-render/block_entity_render.rs): NO-CHANGE (already
  seeded closed in rank2-progress.json; verified this session). All
  eight renderers' typed render bodies are identical between 1.21.4
  and 1.21.5 modulo the 1.21.5 interface changes (7th Vec3 camera
  parameter, mulPose(Quaternionf)->Quaternionfc); translateSignText,
  renderBanner, translateBase and ShulkerBox render likewise. The Rust
  threads the camera in (equivalent to 1.21.4's
  Minecraft.getInstance() lookup); sign_text_near matches
  isOutlineVisible (distance squared < 256.0). LayerDefinitions.createRoots
  delta is 1.21.5-only layers (cold/warm chicken/cow/pig,
  camel/donkey/horse/mule/skeleton/zombie-horse saddle layers, sheep
  wool undercoat) owned by other items; BlockEntityRenderers
  TEST_INSTANCE_BLOCK registration owned by items 163/465. No
  TestInstance arm in the Rust's block_entity_render.rs. Item 91
  (horizon-client/stations.rs): PORTED — smithing trim was broken.
  The Rust required the recipe JSON's "pattern" field (the 1.21.5
  form), but 1.21.4's trim recipes carry no pattern (verified against
  the 1.21.4 server jar's recipe JSONs) — the pattern is resolved from
  the template stack at craft time via TrimPatterns.getFromTemplate
  (scan the trim_pattern registry for the entry whose template_item is
  the template stack's item). All 1.21.4 trim recipes were dropped, so
  smithing trim did nothing. Fix: Smithing::Trim drops the pattern
  field and makes template/addition Optional<Ingredient> (1.21.4's
  fields are all Optional, like the transform's); StationRecipes gains
  trim_patterns loaded from data/minecraft/trim_pattern/*.json
  (template_item -> pattern id); the match resolves the pattern from
  the template stack and trim_demo from the template ingredient's
  first item. SmithingTransformRecipe.assemble already matches 1.21.4
  (transmuteCopy + component patch; matches() identical);
  EnchantingTableBlock SHAPE is identical (1.21.5's
  Block.column(16,12,16) == 1.21.4's box(0,0,0,16,12,16),
  data-driven in the Rust). Tests: the additions-trim-material test
  re-scoped (registry-supplied pattern) and
  smithing_trim_reads_its_pattern_from_the_template_item (new;
  1.21.4-form JSON + trim_pattern/coast.json loads, applies the coast
  trim, and a template with no registry pattern trims nothing) —
  stations suite 26/26 passes. Rank 2: 137/348.
  CI 45851997564 (dispatched Nov 26 ~21:43 NPT on head c3e46cf9):
  completed FAILURE at 12m25s — but differently from runs 45818664429
  and 45823137761. All 11 fixes from the test-health batch are green
  in CI (protocol 425/0, world 447/0, render 2057/0, client 1267/0,
  items, core pass; the two local-only protocol network failures pass
  in CI). The only failures are the 14 horizon-client-lib tests
  shared with the 1.21.5 parent (item_sync x2, lan advancements x5,
  projectiles, realms, save chunks x4, title optimization) — 2417/14,
  identical to the local run and to the parent tree. Next: fix those
  14 (each verified against the 1.21.4 jar, not silently re-scoped)
  before opening more rank-2 items.
  Test-health, round 2 (2026-11-26, after batch 28): crate-tests run
  45851997564 failed only on horizon-client-lib, 2417/14 — exactly the
  14 that fail identically on the 1.21.5 parent tree. All 14 are now
  fixed and verified locally; the lib suite is 2431/0. Each fix was
  verified against the 1.21.4 jar, not silently re-scoped:
  item_sync (2): the painting test used the 26.x component id 109 — the
  Rust's internal id is 89 (1.21.5's PAINTING_VARIANT; 1.21.4 has no
  such component, verified: DataComponents has no painting/sign entries)
  — changed 109 to 89 in the test. The sign test used 26.x ids 118/119;
  1.21.4 has no sign-text components (neither does 1.21.5), the text is
  carried internally under markers -1001/-1002 that write_patch never
  emits — re-scoped to pin that a sign with text writes as a plain sign
  and reads back with no text on it. lan::advancements (5): the book
  writer emitted the 26.2 recipe-display form — Tag slot display plus a
  HolderSet, and ItemStack results item-first. 1.21.4's TagSlotDisplay
  holds a bare tag id (javap: TagKey<Item> in both 1.21.4 and 1.21.5;
  the pinned quirk tag_slot_display_is_id is true) and its
  ItemStack.STREAM_CODEC is count-first (v1_21_4::read_stack reads count
  then item). New slot_display() writes Ingredient.display()'s form
  (tag to Tag+string, one item to Item+id, several to Composite of
  Items; verified against Ingredient.display, displayForSingleItem and
  AbstractCookingRecipe.display in the jar) and the result closures now
  write count first (the owned-content test, count 2 item 1, catches the
  swap). The three furnace tests' expectations moved from Tag(HolderSet)
  to Item(id), the 1.21.4 form. projectiles (1): a real port gap —
  settings() had lost the 26.3 fallback where a registered mod thrown
  type takes its template's (snowball's) tracking settings; restored it
  in lan/tracking.rs. realms (1): test-only — the client already sends
  1.21.4/769/Java 21, the test expected 26.3/777/Java 25. save (4): the
  blending fix was registered at key 4997, past 4189, so it never ran —
  Mojang's DataFixers registers BlendingDataFix at schema 4185
  (between addSchema(4185) and addSchema(4187)), re-keyed to 4185; the
  same wrong 4997 sat in record_is_old, and vanilla's
  IOWorker.isOldChunk compares against sipush 4185 — fixed to 4185.
  The legacy-dimension fallback now sets the overworld for an unknown
  numeric (parseLegacy errors there; the load falls back to overworld).
  The older-records palette assertion moved from `id` to `Name`
  (the grass to short_grass rename fires at fix 3692; the Name to id
  restructure is a 26.x fix, past 4189). title (1): the optimization
  fixture set DataVersion 4950, newer than 4189 — the upgrader never
  downgrades — re-scoped to a 1.20.1 world (3465), which stamps to 4189.
  Local verification after the fixes: horizon-client-lib 2431/0.

  Item 92 (horizon-client/game/lan_entities.rs): NO-CHANGE — verified
  against the 1.21.4 and 1.21.5 named client jars (javap -p -c,
  normalized bytecode). Display: 1.21.5 refactored pos-rot interpolation
  onto InterpolationHandler; 1.21.4 keeps PosRotInterpolationTarget with
  lerpTo/cancelLerp/lerpTarget* (steps = getPosRotInterpolationDuration,
  clamped 0..59). The Rust already implements the 1.21.4 form:
  RemoteTextDisplay.move_to sets steps = teleport_duration.clamp(0, 59)
  and snaps at 0, tick() eases (target - position) / steps per tick;
  text_display.rs has teleport_duration default 0, entity-data id 10,
  clamp 0..59. add/readAdditionalSaveData write the same fields with the
  same defaults in both versions (only the codec API style differs);
  createInterpolatedRenderState is byte-identical. Pinned by the existing
  test metadata_and_position_interpolation_keep_their_separate_clocks
  (duration 2: halfway after one tick, arrived after two).
  MinecartFurnace: no semantic change — 1.21.5's DEFAULT_FUEL (0) and
  DEFAULT_PUSH (Vec3.ZERO) only name 1.21.4's field defaults; the Rust
  save/load (PushX/PushZ doubles, Fuel short, defaults 0) and DATA_ID_FUEL
  (id 13, boolean, true when fuel > 0) already match. TrackedEntity:
  1.21.5 adds broadcastIgnorePlayers and ServerEntity's
  broadcastWithIgnore BiConsumer; broadcast itself is identical, and the
  Rust models neither addition — correct for 1.21.4. Rank 2: 138/348.

  Item 93 (horizon-client-render/collisions.rs): comment-only carry-over.
  The strider liquid shape doc cited `Strider.getLiquidCollisionShape`
  (exists in neither jar; both versions have only
  `Strider.canStandOnFluid`) and `Block.column(16.0, 0.0, 8.0)`
  (1.21.5-only; `Block.column` does not exist in 1.21.4). The shape itself
  is already the 1.21.4 form: `from_cube([1, 2, 1], &[true, false])` is
  the bottom half-box = `LiquidBlock.STABLE_SHAPE`
  (`Block.box(0, 0, 0, 16, 8, 16)`), which
  `LiquidBlock.getCollisionShape` returns for a level-0 source when
  `CollisionContext.canStandOnFluid` (strider: lava) and the context is
  above it. `CollisionContext` placement (1.21.5-only) was already handled
  by item 242; `LiquidBlock.pickupBlock` is body-identical (only the
  parameter type widens, unused). Rank 2: 139/348.

  Item 94 (horizon-client-render/jigsaw_block.rs): comment-only
  carry-over. The two `JointType` docs cited
  `StructureTemplate.getDefaultJointType` (1.21.5-only as a named method);
  in 1.21.4 the rule (front-facing axis horizontal to ALIGNED, else
  ROLLABLE) is inline in `getJointType`, which
  `JointType::default_for_front` already implements exactly.
  `load`/`loadPalette`/`getJointType` differ only by the 1.21.5 NBT API
  refactor. `placeInWorld`'s barrier flags (1.21.4: 20 +
  `Clearable.tryClear`; 1.21.5: 820) belong to item 240 (`processor.rs`
  is in its files list) — NOT fixed here. `placeEntities` `moveTo`
  (1.21.4) vs `snapTo` (1.21.5) is a pure rename: byte-identical bodies.
  Rank 2: 140/348.

  Item 95 (horizon-client/advancements.rs): comment-only carry-over.
  The `potions()` and custom-data docs cited `PotionsPredicate` /
  `CustomDataPredicate.matches` (1.21.5's `core/component/predicates/`
  package, absent in 1.21.4); in 1.21.4 they are
  `advancements/critereon/ItemPotionsPredicate` /
  `ItemCustomDataPredicate`, with the same matching. `NbtPredicate.matches`
  changed signature (ItemStack to DataComponentGetter) with identical
  semantics; `PlayerPredicate` differs only in the `gameMode` access and
  the recipe codec construction; `AdvancementProgress` differs only in
  codec-lambda implementation (`Util.mapValues` vs a manual stream
  collect); `RecipeCraftedTrigger$TriggerInstance` is codec-only.
  Findings NOT fixed here (shared with the 1.21.5 tree, not a delta):
  `potions()` is called only from `BrewedPotionTrigger` with the
  trigger's single-Holder `potion` condition but reads a `v["potions"]`
  HolderSet form, so a specific-potion condition never filters; its
  `effects` check matches no predicate in either version's jar.
  Rank 2: 141/348.

  Item 96 (horizon-client/game/crossbow.rs): PORTED — `CrossbowItem`
  charge/load timing is the 1.21.5 refactor. In 1.21.4 the projectiles
  load in `releaseUsing`, not in `onUseTick` at full charge.
  `tick_crossbow` no longer loads at `now >= 1.0` (start/mid sounds
  unchanged); new `release_crossbow` (from `release_used_item`, slot
  threaded through from both callers) implements 1.21.4's
  `releaseUsing`: `getPowerForTime` = min(held/charge, 1); below 1 loads
  nothing; already-charged returns true; else `load_crossbow`
  (`tryLoadProjectiles`: multishot 3, creative intangible, ammo consumed
  when non-creative) + end sound with pitch `1/(rand*0.5+1)+0.2` at
  volume 1. `getUseDuration` stays 72000: with `useOnRelease` true
  (identical in both versions) the charge+3 duration is unobservable
  client-side. Tooltip: `appendHoverText` (1.21.4-only; 1.21.5 moved it
  to `ChargedProjectiles.addToTooltip`, which the 1.21.4 component
  lacks): first projectile only as `item.minecraft.crossbow.projectile`
  ("Projectile:" per the 1.21.4 lang) + display name; advanced +
  firework rocket appends the rocket's own lines indented with two
  spaces. `Stray.getArrow` is a rename only (`MOVEMENT_SLOWDOWN` to
  `SLOWNESS`, 600 ticks; the Rust already has it).
  `BowItem.releaseUsing` differs only in the `Level.playSound` overload.
  New ignored test `a_crossbow_loads_on_release_not_on_full_charge`
  (needs prepared resources) pins the release-load + early-release
  cases. Rank 2: 142/348.

  Item 97 (horizon-client/game/powder_snow.rs): PORTED — the powder
  snow inside shape is the 1.21.5 rule in the Rust, not 1.21.4's. In
  1.21.4 `BlockBehaviour.getEntityInsideCollisionShape` returns the whole
  block by default and `PowderSnowBlock` does not override it (its
  collision-shape override arrived in 1.21.5), so
  `Entity.checkInsideBlocks` fires `entityInside` on plain whole-block
  overlap. `falling_block.rs` required `fall_distance <= 2.5` or a
  collision with the 0.9-high FALLING top, and `motion.rs` fell through
  to the same shape test when falling fast; both now fire on plain sweep
  overlap (fall distance still reset, stuck multiplier 0.9/1.5/0.9).
  `getCollisionShape` was already the 1.21.4 form (`is_placement` is
  dead — the placement flag is always false in 1.21.4; `fallDistance`
  f64 vs f32 is exact widening, same 2.5 threshold). `entityInside`'s
  client parts are byte-identical; the fire-melt tail is inline in
  1.21.4 vs the applier's runBefore EXTINGUISH in 1.21.5, so the
  `powder_snow.rs` doc cited the 1.21.5 form — fixed. `pickupBlock` is
  body-identical (Player vs LivingEntity param, unused); `fallOn`
  differs only in the float/double param (same 4.0/7.0 sound
  thresholds). Rank 2: 143/348.

  Item 98 (horizon-world/worldgen/generator_decoration.rs): PORTED —
  `WorldGenRegion.setBlock` in 1.21.4 never reads its flags int
  (`setBlockState(pos, state, false)`; the postprocess mark happens
  whenever `hasPostProcess`). 1.21.5 passes the flags through and skips
  the mark when `flags & 16` (UPDATE_KNOWN_SHAPE) is set. The Rust's
  `World::set_block` had the 1.21.5 gate, reachable with bit 16 via the
  structure processor (`processor.rs:1614` passes `flags & !1 | 16` into
  `update_from_neighbour_shapes`), so structure-placed mushrooms,
  magma and soul sand lost their postprocess marks; the gate is removed
  and the flags param is now ignored. The `hasPostProcess` set is
  identical in both jars (brown/red mushroom + magma block; soul sand
  is not in the set in either version, so its `postprocess_above` entry
  is version-independent and left as is). `getBlockEntity` differs only
  in `getString` vs `getStringOr`; `levelEvent`/`playSound` only widen
  Player to Entity; `applyImplicitComponents` differs only in the
  parameter type; `placeSpike` differs only in `moveTo` vs `snapTo`
  (pure rename). Rank 2: 144/348.

  Item 99 (horizon-client-render/block_entity_anim.rs): comment-only
  carry-over. The three renderers' `render` methods are body-identical
  modulo the 1.21.5-only 7th Vec3 camera parameter (slot renumbering
  only; `Quaternionf` to `Quaternionfc` is a JOML API rename).
  `EndGatewayBlock.entityInside` is byte-identical (the 1.21.5 applier
  parameter is unused). `BellBlockEntity.tick` differs only in the
  `Level.playSound` overload. `EnchantingTableBlockEntity`
  load/saveAdditional differ only in the NBT API form. `BlockTags`:
  1.21.5 adds 6 tags (`camels_spawnable_on`,
  `dry_vegetation_may_place_on`, `edible_for_sheep`,
  `plays_ambient_desert_block_sounds`, `replaceable_by_mushrooms`,
  `sword_instantly_mines`) and drops `dead_bush_may_place_on`. One doc
  fix: `ConduitBlockEntity`'s method is `updateDestroyTarget` in BOTH
  jars; the 1.21.4 tree's doc said `updateAndAttackTarget`.
  Findings NOT fixed here (1.21.5-only tags referenced in the 1.21.4
  tree — recorded for their owning items): sheep eat via
  `minecraft:edible_for_sheep` (`goals.rs`, item 88) but 1.21.4's
  `EatBlockGoal` uses `IS_TALL_GRASS` = `BlockStatePredicate.forBlock`
  (SHORT_GRASS) or GRASS_BLOCK below, so sheep never eat short grass
  here; camels spawn via `minecraft:camels_spawnable_on`
  (`spawn_rules.rs`, item 88) but 1.21.4 has no `checkCamelSpawnRules`
  and no CAMEL entry in `SpawnPlacements` (camels spawn only via desert
  village structures), so camels should never naturally spawn here;
  huge mushrooms replace via `minecraft:replaceable_by_mushrooms`
  (`tree_blocks.rs`/`vegetation.rs`, items 252/253) but 1.21.4's
  `AbstractHugeMushroomFeature` uses `isAir || is(LEAVES)` in its space
  check and no tag in `place`, so mushrooms only replace air here.
  Also: `block_entity_nbt.rs` writes `CustomName` via the 1.21.5 NBT
  codec form (`storeNullable`, `ComponentSerialization.CODEC`) while
  1.21.4 writes a JSON string (`Component$Serializer.toJson`) —
  recorded for the block-entity-NBT items (notably item 163).
  Rank 2: 145/348.

  Item 100 (horizon-client-render/options_file.rs): PORTED —
  `Options.startedCleanly` is 1.21.5-only: neither 1.21.4's `Options`
  nor its `Minecraft` class contains the field (no `startedCleanly`,
  `resetting fullscreen`, or `unexpected shutdown` strings). The Rust's
  unclean-start recovery (init: read flag, set false, save;
  `onGameLoadFinished`: set true, save; leave fullscreen + clear
  `fullscreenResolution` when false) is the 1.21.5 mechanism; removed
  entirely: the `startedCleanly` entry from the options TABLE,
  `unclean_start_recovery`, the `game.rs` init block and both
  `onGameLoadFinished` markers, and the `core_tests` test that pinned
  the 1.21.5 behavior (a 1.21.5-behavior test, not a 1.21.4 one). The
  two `save()` calls are kept as plain persistence points. Everything
  else in `Options` is NBT-API-only: `dataFix`, `load`'s
  fancyGraphics-to-graphicsMode migration and `fullscreenResolution`
  store, `save`'s `fullscreenResolution` write (the Rust keeps the
  string verbatim and writes it back — the 1.21.4 form in effect),
  `processOptions` (identical except the `startedCleanly` line),
  constructor (`soundSourceVolumes` EnumMap factory form only).
  Rank 2: 146/348.

  Item 101 (horizon-client/game/falling_damage.rs): carry-over, one doc
  fix. `Damage::from_nbt` is already the 1.21.4 read (`HurtEntities` or
  the anvil default, `FallHurtAmount` or 0.0, `FallHurtMax` or 40 —
  equivalent to 1.21.5's `getBooleanOr`/`getFloatOr`/`getIntOr` with the
  anvil default); `impact()` is `Mth.ceil(fallDistance - 1.0F)` with the
  same floor/min/max and `0.05 + steps * 0.05` wear chance;
  `wear_anvil`/cancelDrop match; `finish_falling_block`'s `TileEntityData`
  merge is 1.21.4's `getAllKeys` copy loop (1.21.5's `forEach` copies the
  same tag per key); `causeFallDamage` returns false at the end in 1.21.4
  and the Rust consumes no boolean. The float-vs-double fallDistance
  param is provably the same ceil for float inputs. Doc fix:
  `finish_falling_block` cited 1.21.4's tick as 1.21.11. Known gap, NOT
  fixed here: the tree stores `Entity.fallDistance` as f64 everywhere
  (the 1.21.5 form; 1.21.4's field is float) — narrowing it is a
  tree-wide decision, like the `level/mod.rs` flags question.
  Rank 2: 147/348.

  Item 102 (horizon-client/mobs/bee.rs): PORTED — `Bee.customServerAiStep`
  in 1.21.4 reads `Entity.isInWaterOrBubble` (water, or the bubble column
  at the bee's feet — `Entity.isInBubbleColumn` reads the block state at
  the entity's block position); 1.21.5 narrowed it to `isInWater`. The
  Rust used `m.fluids.in_water` only, so a bee in a bubble column never
  drowned here; `server_ai_step` now also checks the block at the bee's
  feet for `minecraft:bubble_column`, with a regression test (5 ticks in
  a column count `under_water` to 5; out of it resets to 0).
  `isNightOrRaining` is NOT a delta: 1.21.4 `isNight()` equals 1.21.5
  `isDarkOutside()` (`isDay()` and `isBrightOutside()` are byte-identical:
  `!hasFixedTime() && skyDarken < 4`), and the Rust's `bees_stay_in_hive`
  day-time range 12542..23460 reproduces `skyDarken >= 4` exactly
  (verified numerically against `Level.updateSkyBrightness` with
  `Mth.cos`'s table and f32 rounding); its doc cited the non-existent
  `EnvironmentAttributes.BEES_STAY_IN_HIVE`, fixed to cite
  `Bee.isNightOrRaining`. `TorchflowerCropBlock` is identical:
  `getBonemealAgeIncrease` returns 1 in both; `getShape`'s `SHAPES` vs
  `SHAPE_BY_AGE` hold the same boxes (age 0: 5,0,5,11,6,11; age 1:
  5,0,5,11,10,11 — 1.21.5 builds them via `Block.column(6, 0, 6 +
  age*4)`); add/readAdditionalSaveData differ only in the NBT API form.
  Rank 2: 148/348.

  Item 103 (horizon-client/mobs/zombie_villager.rs): PORTED —
  `ZombieVillager.readAdditionalSaveData` calls `startConverting(player,
  time)` when `ConversionTime` is present, re-applying the cure's strength
  for the remaining time; the Rust's `read_nbt` set the cure but not the
  effect, so a loaded converting zombie villager lost its strength.
  `read_nbt` now adds `minecraft:strength` for the cure's remaining time
  (and drops weakness, as `startConverting` does) after the saved
  `active_effects` are loaded, with a round-trip test (a saved cure comes
  back with strength 4000). `startConverting` itself already matches:
  `DAMAGE_BOOST`'s registry id is `minecraft:strength` in 1.21.4 (the
  1.21.5 change was a field rename only); weakness removed; entity event
  16. `finishConversion`'s lambdas are identical modulo numbering
  (`!EnchantmentHelper.has` `PREVENT_ARMOR_CHANGE`;
  `dropPreservedEquipment` moves cursed stacks to villager slots
  index+300 — the Rust's `convert_to_villager` matches, verified against
  `Mob.dropPreservedEquipment`); `defineSynchedData`'s
  `createDefaultVillagerData()` is `new VillagerData(PLAINS, NONE, 1)`;
  `finalizeSpawn`'s byBiome/setType equals withType; `setVillagerData`
  clears offers on a profession change (`villager.rs`);
  `VillagerData`/`Offers`/`Xp` load in `entity_nbt.rs`. One comment fix:
  the strength amplifier is `Math.min(difficulty - 1, 0)`, 0 except on
  Peaceful where vanilla gives -1 (3 less melee damage) — this keeps 0
  everywhere. Rank 2: 149/348.

  Item 104 (horizon-client-render/block_entities.rs): carry-over.
  The ticker-visible behaviour of `LevelChunk` is identical in both
  versions. `postProcessGeneration` differs only in jump offsets;
  `clearAllBlockEntities` is identical; `promotePendingBlockEntity`
  differs only in `getString` vs `getStringOr` (same `DUMMY` check);
  `replaceWithPacketData` decodes the heightmaps in the packet reader in
  1.21.5 vs from the `CompoundTag` in 1.21.4, same data, and its
  `BlockEntityTagOutput` lambda is identical (`getBlockEntity(IMMEDIATE)`,
  type check, `loadWithComponents`). The `setBlockState` differences are
  all outside this file's ticker bookkeeping: 1.21.4 takes a boolean and
  calls `oldState.onRemove` server-side unconditionally, while 1.21.5
  takes flags and calls `preRemoveSideEffects` (unless flags & 256),
  `affectNeighborsAfterRemoval` for rails, and `onPlace` only when
  !(flags & 512) — the 1.21.5-only flags
  (`UPDATE_SKIP_BLOCK_ENTITY_SIDEEFFECTS` 256, `UPDATE_SKIP_ON_PLACE`
  512) already present in `level/mod.rs`, whose 1.21.5 form is a known
  open parent decision (item 52). The block-entity create/remove/mismatch
  path inside `setBlockState` is identical in both jars.
  Rank 2: 150/348.

  Item 105 (horizon-client-render/command_block.rs): PORTED.
  `BaseCommandBlock.save` in 1.21.4 writes `LastOutput` as a JSON string
  (`Component$Serializer.toJson`) and `LastExecution` only while
  `updateLastExecution && lastExecution > 0`; the Rust wrote the codec
  compound and any value but -1. `save` now writes
  `Tag::String(serde_json::to_string(component))` and skips
  `LastExecution` at 0, compacting a style-less literal to the JSON string
  itself (`toJson` of the literal is `"hi"`, not `{"text":"hi"}` — the
  first save wrote the object form and the minecart round-trip test caught
  it). `load` reads `CustomName`/`LastOutput` only from
  String tags (1.21.4's `getString` drops a compound) and parses the JSON
  before `comp_text`: a malformed `LastOutput` becomes a literal of the
  parse error, as vanilla's `catch (Throwable)` does, and an unparsable
  `CustomName` is no name (`parseCustomNameSafe`); `name()` parses the
  stored JSON string the same way. `load` otherwise matches: `track_output`
  and `update_last_execution` default true, `LastExecution` -1 unless
  present and updating. `CommandBlockEntity.loadAdditional` is NBT-API-only
  noise (`getBooleanOr` vs `getBoolean`, same false defaults); the 1.21.5
  constructor's explicit powered/auto/conditionMet false inits are field
  defaults in 1.21.4; `isAutomatic`/`onModeSwitch`/`setAutomatic` differ
  in spacing only. Test
  `components_save_as_json_strings_and_zero_execution_is_not_kept` covers
  the styled round-trip, the skipped zero, and the dropped compound forms.
  Follow-up: crate-tests 45892679369 failed on
  `a_command_cart_saves_its_base_command_block`, whose fixture fed a bare
  non-JSON `LastOutput` string and expected it back verbatim (the old
  1.21.5-form literal read). 1.21.4's load calls the strict
  `Component$Serializer.fromJson` (`JsonParser.parseString`), so a bare
  string loads as a literal of the parse error; a vanilla-written save
  never holds one (`toJson` quotes it). The fixture now uses the JSON-string
  form. Rank 2: 151/348.

  Item 106 (horizon-client/crafting.rs): PORTED, two fixes. (1) The
  special-recipes loader gap: all 11 1.21.4 special JSONs carry only
  `type` + `category`, and the loader read ingredients from the JSON and
  dropped them; three types (`crafting_special_armordye`,
  `crafting_special_mapcloning`, `crafting_special_tippedarrow`) had no
  loader arm at all, so armor dyeing, map cloning and tipped arrows could
  not be crafted. Ported the 1.21.7 fix (b680e303) with every hardcoded
  input verified against the 1.21.4 server jar: `ing_or`/`result_or`/`dyes`
  fallbacks; armor dye targets `#minecraft:dyeable` with the sixteen dyes
  (1.21.4 has no `#minecraft:dyes` tag); banner duplication `#minecraft:banners`
  (vanilla tests `instanceof BannerItem`); shield decoration `#minecraft:banners`
  + shield; book cloning source `written_book` + material `writable_book`
  (1.21.4 has NO `#minecraft:book_cloning_target` tag — the 1.21.7
  fallback would have resolved to an empty ingredient here); firework
  rocket paper/gunpowder/firework_star, 3 out (assemble `iconst_3`);
  firework star `SHAPE_BY_ITEM` (large_ball fire_charge, burst feather,
  star gold_nugget, creeper `#minecraft:skulls`) with gunpowder/diamond/
  glowstone_dust; star fade firework_star; map extending filled_map +
  paper (`Items.MAP` in `MapExtendingRecipe` is only the `ShapedRecipe`
  dummy result); map cloning as a transmute, filled_map + 1..8 map,
  `add_count`; decorated pot `#minecraft:decorated_pot_ingredients`;
  tipped arrow as an imbue, lingering potion + arrow, 8 out (assemble
  `bipush 8`). `BookCloningRecipe` itself already matched
  (`WRITTEN_BOOK`/`WRITABLE_BOOK` items; `tryCraftCopy` caps generation
  at 2, which the Rust's `[0, 1]` bounds default reproduces; remaining
  items stop at the first written book). Test
  `special_recipes_load_from_type_and_category_only_jsons` loads all 11.
  (2) `DyedItemColor`: 1.21.4's record is `(rgb, showInTooltip)` with
  `CODEC` the record plus a bare-int alternative and `STREAM_CODEC`
  int+bool; the Rust kept a bare `i32`, so a compound `dyed_color`
  decoded to `None` (the colour was lost) and the line could never hide.
  `StackData` gains `dyed_color_hidden` (default shown); the decode
  accepts an int or `{rgb, show_in_tooltip}` in both decode paths; the
  encode writes the record when hidden; `dyed_line` gates on it (the
  packet path already hid it through `TooltipDisplay`); `applyDyes`
  preserves the flag, as vanilla does (it keeps the existing
  `showInTooltip`); the cauldron wash clears it. Only the loot
  `ToggleTooltips` function and explicit components set it false.
  `Recipe.recipeBookCategory` is static-init only (`KEY_CODEC` added in
  1.21.5): no change. Rank 2: 152/348.

  Item 107 (horizon-client/mobs/ravager.rs): carry-over.
  `CropBlock.entityInside` and `PitcherCropBlock.entityInside` are the
  same trample in both versions (ServerLevel + Ravager + mobGriefing ->
  `destroyBlock(pos, true, entity)`; the super call is a no-op — 1.21.4
  has no `VegetationBlock`, and neither `BushBlock` nor `DoublePlantBlock`
  defines `entityInside`). The Rust's deflated-bounds sweep breaks
  wheat/carrots/potatoes/beetroots/torchflower_crop/pitcher_crop with drops
  under mobGriefing, matching. The shape deltas are real but NOT in Rust
  source: 1.21.4's `PitcherCropBlock` keeps per-age
  `UPPER_SHAPE_BY_AGE` (upper: ages 0/1/2/4 `box(3,0,3,13,15,13)`, age 3
  `box(3,0,3,13,11,13)`) while 1.21.5 computes the upper half as
  `column(w, 0, max(0, v2-16))` (ages 0/1 empty, age 2
  `box(3,0,3,13,1,13)`, age 3 `box(3,0,3,13,12,13)`, age 4
  `box(3,0,3,13,16,13)`); the lower halves and both collision shapes
  (`BULB box(5,-1,5,11,3,11)`, `CROP box(3,-1,3,13,5,13)`) are identical,
  and 1.21.4's upper-half collision falls back to
  `BlockBehaviour.getCollisionShape` (the visual shape) while 1.21.5
  returns `Shapes.empty()` for the upper half. Shapes live in the
  prepare-derived `cached-collision.nbt` (`VanillaCollisionOracle` vs the
  1.21.4 jar), so the 1.21.4 tree gets the 1.21.4 shapes by construction.
  `CropBlock`'s `SHAPE_BY_AGE` is identical in both
  (`box(0,0,0,16,2+2*age,16)`, ages 0..7). Rank 2: 153/348.

  Item 108 (horizon-protocol/buf.rs): carry-over. `ByteTag`/`EndTag`/
  `StreamCodec` behaviour is identical in both versions. `ByteTag` is a
  plain class in 1.21.4 (field `data`, package-private ctor) vs a Record
  in 1.21.5 (field `value`); the wire form is one byte either way, the
  `valueOf` cache and `write` are identical, `getAsX` vs `xValue` are
  renames only. `EndTag.getAsString` differs only in construction (1.21.4
  returns `getAsString`, 1.21.5 builds via
  `StringTagVisitor.visitEnd`) with the same empty result.
  `StreamCodec.composite` overloads (1..8 fields) differ only in
  anonymous-class numbering. Verified the constants the Rust cites:
  `EndTag.sizeInBytes()` = 8 (`MIN_TAG_COST`), `ByteTag.sizeInBytes()` =
  9, and `FriendlyByteBuf.readNbt` passes
  `NbtAccounter.create(2097152L)` in both jars. The `packet!` macro
  mirrors `composite` (fields in order); the NBT limits already match.
  Rank 2: 154/348.

  Item 109 (horizon-protocol/secure_chat.rs): carry-over, comment-only.
  1.21.5 added a checksum byte to `LastSeenMessages$Update` (2-arg ctor ->
  3-arg, `read`/`write` gain a byte, new `verifyChecksum`/`checksum`/
  `computeChecksum`); the Rust already has the 1.21.5 form and the wire
  delta is adapter-handled (`adapter/translate.rs` strips the last byte
  serverbound for `<= V1_21_4`; `adapter/client_session/step_1_21_4.rs`
  pushes 0, which 1.21.5 reads as "not checked"). Validator semantics
  match: 1.21.4 `applyOffset` returns boolean and `applyUpdate` returns
  `Optional` (empty on failure), 1.21.5 throws `ValidationException`; the
  Rust returns `Result<Vec<Signature>, String>`, matching both. The server
  failure path matches in both: disconnect with
  `CHAT_VALIDATION_FAILED`; 1.21.4 `unpackAndApplyLastSeen` warns the line
  with no stack trace. The comment in `lan/hub/chat.rs` said vanilla logs
  the exception's stack trace; corrected to the 1.21.4 form. Rank 2:
  155/348.

  Item 110 (horizon-assets/item_model.rs,
  horizon-client/item_render/special.rs): PORTED. The 1.21.5-only deltas
  are `RegistryContextSwapper` threading (`ClientItem.withRegistrySwapper`,
  `BakingContext` 4-arg ctor + `contextSwapper()`,
  `ConditionalItemModel$Unbaked.adaptProperty`/`swapContext`),
  `ModelSelector` wrapping `SelectItemModel`'s case map, and
  `SpecialModelWrapper` taking `ModelRenderProperties` (`getProperties`)
  plus setting extents in `update`. None exists in the Rust (no swapper or
  render properties anywhere; `Select` is a plain case list; `Special`
  decodes its fields directly) -- the 1.21.4 form. Real gap found and
  fixed: the Rust's `SpecialKind` had no `minecraft:bed` arm although
  vanilla 1.21.4 registers it in `SpecialModelRenderers` and all 16 bed
  item models use it (`black_bed.json`: special with base
  `item/black_bed`, model `{type: bed, texture: minecraft:black}`), so bed
  items failed to decode. Added `SpecialKind::Bed { texture }` (decode +
  `to_json`), `bed_head`/`bed_foot` layers (`ModelLayers.BED_HEAD`/
  `BED_FOOT`, present in the prepare-exported `entity-models.json` via
  `LayerDefinitions.createRoots`), and render + extents arms following
  `BedRenderer.renderInHand` (head then foot, both facing south, translate
  `(0, 0.5625, z)` with z 0 / -1, X +90 deg; the 180+180 Z turn is
  identity; `Sheets.createBedMaterial` prefixes `entity/bed/`). Tests:
  `a_bed_special_model_decodes_its_colour_texture` (horizon-assets) and
  `a_bed_item_draws_its_head_and_foot_in_the_beds_colour`
  (horizon-client). Known gap, not fixed: `standing_sign`/`hanging_sign`
  special types are registered in vanilla but have no Rust arm -- no
  vanilla item uses them (signs use plain models), so only a pack
  referencing them fails to decode. Rank 2: 156/348.

  Item 111 (horizon-client/lan/hub.rs): carry-over.
  `placeNewPlayer`/`respawn`/`sendAllPlayerInfo` differ only by
  non-behavioural changes. `Entity.moveTo(DDDFF)` was renamed `snapTo` in
  1.21.5 (bodies byte-identical: `setPosRaw`/`setYRot`/`setXRot`/
  `setOldPosAndRot`/`reapplyPosition`; same for the `Vec3` overload).
  `Inventory.getSelectedSlot()` in 1.21.5 is the `selected` field in
  1.21.4. The ender-pearl/parent-vehicle load moved into
  `Optional.ifPresent(lambda$placeNewPlayer$1)` in 1.21.5; the lambda body
  calls exactly `loadAndSpawnEnderPearls(tag)` +
  `loadAndSpawnParentVehicle(tag)`, the same two calls 1.21.4 makes
  directly. The protocol-bind refactor (`UnboundProtocol.bind(Function,
  Object)` vs `ProtocolInfo$Unbound.bind(Function)`) is network-layer
  only. The Rust's `place_new_player`/`respawn`/`sendAllPlayerInfo`
  already match 1.21.4 (`SetHeldSlot` from `g.selected`; respawn sets
  position + previous position + yaw/pitch, citing `snapTo`). Known gap,
  not a delta: the host does not re-spawn a joining player's saved thrown
  ender pearls or parent vehicle (vanilla does in both versions). Rank 2:
  157/348.

  Item 112 (horizon-client/mobs/shulker.rs): carry-over.
  `hitByShulkerBullet` differs only by `moveTo(Vec3)` -> `snapTo(Vec3)`
  (bodies identical, both delegate to the 3-arg form). `setupRotations`
  differs only by `rotateAround(Quaternionf)` -> `(Quaternionfc)`
  signature. `defineSynchedData`: `DEFAULT_ATTACH_FACE` constant vs
  `Direction.DOWN` literal. `addAdditionalSaveData`: 1.21.5 stores
  `AttachFace` via `LEGACY_ID_CODEC`, 1.21.4 `putByte(get3DDataValue)` --
  the Rust already writes `Tag::Byte` (`entity_nbt.rs`, the 1.21.4 form)
  and reads any number, clamped 0..5. `hit_by_shulker_bullet` matches
  vanilla: teleport, other-shulkers count excluding self (= vanilla's
  `i - 1`), failure chance others/5.0 on the level random, baby of the
  same colour spawned at the old position. `rotations()` matches
  `setupRotations` (rotY(PI) + `rotateAround` of the opposite-face
  rotation about (0, 0.5, 0)); all six `face_quat` arms verified against
  `Direction.getRotation()` of the opposite face in the 1.21.4 jar. The
  `getInterpolation`/`lerpTo` and component `get`/
  `applyImplicitComponents` reshuffles change no shulker behaviour.
  Rank 2: 158/348.

  Item 113 (horizon-assets/item_model.rs): ported.
  1.21.5 refactored `DataComponentPredicate` into an interface in
  `core.component.predicates` with `singleCodec(String)` and a dispatched
  `STREAM_CODEC` on the new `DATA_COMPONENT_PREDICATE_TYPE` registry;
  1.21.4 has it as a final class in `core.component` holding a
  `List<TypedDataComponent>`, `CODEC` a `VALUE_MAP_CODEC` map and
  `STREAM_CODEC` a plain list of `TypedDataComponent.STREAM_CODEC`.
  Neither the `component` condition property nor the `component` select
  property exists in 1.21.4 (`ConditionalItemModelProperties` and
  `SelectItemModelProperties` gain `ComponentMatches` only in 1.21.5,
  verified in both jars' registries), but the Rust decoded the 1.21.5
  `singleCodec` `{predicate, value}` form for both, so the item-model
  parity harness disagreed with the 1.21.4 oracle (Rust ok=true, oracle
  false). Removed the `Component` variants + decode/resolution arms from
  `ConditionProperty` and `SelectProperty`, deleted
  `item_model/component_predicate.rs`, removed
  `ItemEnv::component_matches` and `ItemEnv::enchantment_in_tag` and the
  game's `enchantment_in_tag` override in `game/item_draw.rs`; both
  properties now fall through to the unknown-property errors, matching
  vanilla 1.21.4's codec rejection. `MinMaxBounds`/`Doubles`/`Ints` are
  identical between versions; `EnchantmentsPredicate` (+2 nested classes)
  is 1.21.5-only. Test:
  `the_1_21_5_component_properties_do_not_decode_in_1_21_4`. Recorded,
  not fixed (needs its own focused item): the protocol-side
  `DataComponentPredicate` handling is systematically 1.21.5-shaped --
  `horizon-protocol/src/item/skip.rs::component_matchers`,
  `item/hash/gameplay.rs::partial_predicate`,
  `item/hash/raw.rs` (`data_component_predicate_type` id->name table
  0-13), `lan/guest.rs` and `horizon-command/src/item.rs` reference the
  registry; 1.21.4 has no `DATA_COMPONENT_PREDICATE_TYPE` registry
  (verified in `BuiltInRegistries` for both jars) and its `STREAM_CODEC`
  is a plain list of `TypedDataComponent`s (type id + value, no boolean,
  no predicate-type registry). Rank 2: 159/348.

  Item 114 (widely named: network/chat): ported.
  1.21.5 refactored `ClickEvent` into a sealed interface with per-action
  records and per-action codec fields (`url`/`command`/`page`/`path`),
  and renamed the `Style` fields to `click_event`/`hover_event`; 1.21.4
  has a single final `ClickEvent` (`Action`, `String value`) whose
  `CODEC` is `action` + `Codec.STRING fieldOf("value")` for ALL actions
  (`change_page`'s value is the page as a string), and
  `Style$Serializer` uses `clickEvent`/`hoverEvent` (verified by javap
  on both jars). The Rust's parse side already handled both forms, but
  its encode side (`Text::component` via `style_json`) wrote the 1.21.5
  form, so a component sent to vanilla 1.21.4 lost its click event (an
  optional field, silently dropped). Fixed the encode to `{action,
  value}` for every action and to `clickEvent`/`hoverEvent` keys, in
  `text.rs` (`style_json` + `STYLE_KEYS`), `scoreboard.rs`
  (`canonical_component` arm), `signs.rs` (`STYLE_KEYS`, `message()`
  fallback, `click_event()` reader -- sign lines are saved as
  `SignText.CODEC`), `commands.rs` + `commands/players.rs` +
  `lan/hub.rs` (player display names; `with_component_fields` is sent to
  vanilla), `lan_trading.rs` (NBT `Tag` key), `advancements.rs` (reads
  the tree's own `component()` output), `lan/conn.rs` (component key
  `ORDER` list), and the `horizon-command` component validator
  (`CLICK_ACTIONS` now require `"value"` per action, key lookup accepts
  both spellings). Tests:
  `click_events_serialize_and_inherit_as_one_style_field` extended with
  `OpenUrl`/`ChangePage` cases. Carry-over, no delta: `resolveStyle`
  is the same logic in both (only the `HoverEvent` representation
  differs); `flatCodec(int)` is `flatCodec(Integer.MAX_VALUE)` in
  1.21.4, so no size limit (1.21.5's `flatRestrictedCodec`/`isTooLarge`
  is new); `MessageSignature` checksum/describe are 1.21.5-only and
  already adapter-handled (item 109). `COPY_TO_CLIPBOARD`
  `allowFromServer` is true in 1.21.4, matching the Rust. Recorded, not
  fixed (pre-existing, identical in both versions): `show_entity` is
  written as `{id: entity type, uuid, name}` but vanilla (both versions)
  wants `{type, id: UUID, name}`; the Rust's form round-trips through
  its own parse, so player-name hovers display fine locally and it only
  breaks if such a component is sent to vanilla. Rank 2: 160/348.

  Item 115 (horizon-client-render/mesher.rs): ported.
  `BlockElement` is a record in 1.21.5 (`Vector3fc` + accessors) vs a
  class with public `Vector3f` fields in 1.21.4; the `Deserializer`
  differs only in the ctor signature -- no behavioural delta, and the
  Rust already implements 1.21.4's `fillUvs` exactly (`bake.rs`
  `default_uv` matches all six `uvsByFace` formulas, verified against
  the 1.21.4 jar). The real deltas are in `BlockColors.createDefault`:
  1.21.5 added `LEAF_LITTER` with the new dry-foliage tint
  (`getAverageDryFoliageColor`, fallback `0xFF5C3C32`), added `BUSH` to
  the grass-tint set and `WILDFLOWERS` to the pink-petals set -- all
  three blocks are 1.21.5-only (no `LEAF_LITTER`/`WILDFLOWERS`/`BUSH`
  block in the 1.21.4 jar). Removed the three dead arms from the mesher
  tint table (`bush` from the grass set, `wildflowers` from
  `pink_petals`, `leaf_litter` -> `DryFoliage`) and the `leaf_litter` ->
  `Tint::Dry` arm + the now-never-constructed `Tint::Dry` variant from
  `lod/mesh.rs`; the table now matches 1.21.4's `createDefault` exactly
  (stems keep the age formula, verified: `rgb(age*32, 255-age*8,
  age*4)`; attached stems keep constant `0xE0C71C`; lily pad, redstone,
  sugar cane, spruce/birch constants unchanged). Dry-foliage support
  machinery (`Tint::DryFoliage`, `Resolver::Dry`, the biome
  `dry_foliage` field, the Polytone `dry_foliage` function) stays: it is
  data-driven and no 1.21.4 block uses it, and removing it is a
  cross-crate refactor out of proportion to this item. Tests: `cargo
  test -p horizon-client-render --lib mesher`, 68 passed. Rank 2:
  161/348.

  Item 116 (horizon-client/game/wind.rs): carry-over.
  `FenceGateBlock`'s shape constants differ only in construction --
  1.21.5 builds `cube(16,16,4)` + `Shapes.rotateHorizontalAxis` into
  `SHAPES`/`SHAPES_WALL`, 1.21.4 uses explicit `Z_SHAPE` (0,0,6 ->
  16,16,10), `X_SHAPE` (6,0,0 -> 10,16,16), `Z_SHAPE_LOW`/`X_SHAPE_LOW`
  (y max 13); the geometry is identical. `onExplosionHit`,
  `neighborChanged` and `useWithoutItem` differ only by the `playSound`
  signature widening (1.21.5 `Level`/`ServerLevel.playSound` takes
  `Entity`, 1.21.4 takes `Player`; both call sites pass null). The
  Rust's `fence_gate_exploded` (`horizon-world` `level/interact.rs`)
  matches 1.21.4's `onExplosionHit` exactly: returns unless the
  explosion can trigger blocks and the gate is unpowered, toggles `OPEN`
  via `setBlockAndUpdate`, plays `fenceGateClose`/`Open` at volume 1.0
  and pitch `random*0.1+0.9`, emits `BLOCK_CLOSE`/`BLOCK_OPEN` with no
  source entity, and keeps the facing (unlike a click).
  `WindChargeItem.use` differs only by the same `playSound` signature;
  `throw_wind_charge` matches (`PROJECTILE_SHOOT_POWER` 1.5f verified in
  the 1.21.4 jar, sound `WIND_CHARGE_THROW` volume 0.5 pitch
  `0.4/(random*0.4+0.8)`, `ITEM_USED` stat, consume 1 unless creative).
  `EntityRenderState` gains only the 1.21.5 `fillCrashReportCategory`
  crash-report helper, no behaviour. Recorded, not fixed (pre-existing,
  identical in both versions): the wind-charge throw sound plays at the
  eye position in the Rust vs the player's feet position in vanilla.
  Rank 2: 162/348.

  Item 117 (horizon-client/mobs/finalize.rs): carry-over.
  `VillagerType.byBiome` maps 29 biomes to 7 types identically in both
  versions (verified pair by pair against the 1.21.4 jar; only the value
  type differs -- 1.21.5 stores `ResourceKey<VillagerType>`, 1.21.4 the
  type directly) and the Rust's `villager_type` matches all 29 pairs
  exactly (plains 0, desert 1, jungle 2, savanna 3, snow 4, swamp 5,
  taiga 6). `PatrollingMonster` differs only by codec-form reshuffles
  (`storeNullable`/`read` vs `NbtUtils.writeBlockPos`/`readBlockPos`,
  `getBooleanOr(key, false)` vs `getBoolean(key)` -- same NBT, same
  results; the ctor's explicit false inits are Java field defaults).
  `AttributeModifier` is the record conversion + whitespace only.
  `Spider$SpiderEffectsGroupData.setRandomEffect` differs only by
  effect field renames (1.21.5 `SPEED`/`STRENGTH` vs 1.21.4
  `MOVEMENT_SPEED`/`DAMAGE_BOOST` -- same registry ids
  `minecraft:speed`/`minecraft:strength`); the Rust's `spider()`
  matches (`int(5)`: 0|1 speed, 2 strength, 3 regeneration, else
  invisibility, hard + 10% chance, duration -1 amplifier 0).
  `DataComponentGetter` and `SpawnPrioritySelectors` are 1.21.5-only
  classes (absent from the 1.21.4 jar); the Rust needs neither -- the
  mooshroom variant goes through `species_variant_component` and variant
  spawn conditions are read straight from the variant JSONs'
  `spawn_conditions` (`VariantRegistry`). Rank 2: 163/348.

  Item 118 (horizon-client/examples/probe/mob_ai_probe.rs): carry-over.
  `serverTick`/`addOccupant`/`releaseOccupant` differ only by the
  `playSound` signature widening (`Entity` vs `Player`) and
  `moveTo`->`snapTo`. The Rust's `beehive.rs` already matches 1.21.4:
  `IGNORED_BEE_TAGS` is exactly the 1.21.4 30-entry list in order
  (verified against the jar; 1.21.5 merged `ArmorDropChances`+
  `HandDropChances` into `drop_chances` and renamed `FallDistance` to
  `fall_distance`, 29 entries). `loadAdditional`/`saveAdditional` are
  codec-form reshuffles (`CompoundTag.read` vs `NbtOps` parse,
  `NbtUtils.readBlockPos` vs `BlockPos.CODEC`) with identical NBT
  (`bees` + `flower_pos`). The `BEES` component is a raw list in
  1.21.4 vs the `Bees` record in 1.21.5; the Rust writes `Tag::List`
  (`beehive.rs` `to_nbt`, `block_entity_nbt.rs`) -- the 1.21.4 form.
  `applyImplicitComponents`/`collectImplicitComponents` differ only by
  the `Bees` record wrap. Rank 2: 164/348.

  Item 119 (horizon-items/items.rs): ported.
  1.21.5 added `can_destroy_blocks_in_creative` (bool, default true)
  as a 4th `Tool` record field -- in both the `CODEC`
  (`optionalFieldOf`) and the `STREAM_CODEC` (trailing bool) -- and
  added `Item.canDestroyBlock` (a tool with the field false blocks
  destruction for instabuild players). Neither exists in 1.21.4
  (verified by javap: no `canDestroyBlock` on `Item`, 3-field `Tool`).
  Removed `can_destroy_blocks_in_creative` from `ItemInfo` + the
  loader, removed `can_destroy_block` and its guards in `game.rs`
  (`break_block_as_player`, mining tick), `game/remote.rs`
  (`remote_mining`) and `game/pad/guide.rs` (`breakable` chain); in
  1.21.4 creative always destroys. Rewrote the creative-mining harness
  for 1.21.4: the probe prints true per row and the oracle prints true
  per row (the 1.21.5 oracle used the 4-arg `Tool` ctor +
  `canDestroyBlock` and cannot compile against the 1.21.4 jar); the
  driver now reads `items.json` from `lab.pinned_version()` instead of
  the hardcoded 26.3 path. `FireChargeItem.playSound` /
  `FlintAndSteelItem.useOn` differ only by the `playSound` signature;
  `fire_starter_pitch` already matches both formulas (fire charge
  `(next-next)*0.2+1.0`, flint `next*0.4+0.8`). The wire side is
  adapter-handled (`horizon-protocol` `v1_21_4.rs` strips the trailing
  `Tool` bool; `skip.rs` reads the 1.21.5 layout). Incidental
  pre-existing fix in the same commit: `sprint_probe.rs` omitted the
  `Conditions.vehicle_on_ground` field, so the `client_probe` example
  did not compile on this branch; set to false (the sprint oracle's
  mock vehicle never reports onGround). Tests: horizon-items lib 111
  passed; `client_probe` example compiles. Rank 2: 165/348.

  Item 120 (horizon-client/game/lan/level_event_targets.rs): carry-over.
  `LevelAccessor` differs only by the `Entity`-vs-`Player` widening on
  `levelEvent`/`playSound` (both abstract and default overloads; the
  3-arg `levelEvent` delegates to the 4-arg with null in both) and by
  two empty default methods -- `blockUpdated` (1.21.4-only) and
  `updateNeighborsAt` (1.21.5-only) -- which both just return.
  `level_event_targets.rs` correctly models the null-source delegation
  (`level_event_except`: the listed ids include the acting player);
  event 1500 verified in the 1.21.4 jar (`ComposterBlock` calls
  `Level.levelEvent(int, BlockPos, int)`). Recorded, NOT fixed (needs
  its own focused item): the `TestBlock`/`TestInstanceBlock` feature
  (`test_block.rs`, `BlockEntity::TestBlock` wiring, `block_editor`
  `open_test_block`, `game_tests` `test_instance_block` references) is
  1.21.5-only -- neither class exists in either 1.21.4 jar;
  `test_block.rs` cites `level.updateNeighborsAt`, the 1.21.5-only
  method. Rank 2: 166/348.

  Item 121 (horizon-client/text_display.rs): carry-over.
  `Brightness.pack` in 1.21.4 is inline (`block << 4 | sky << 20`);
  1.21.5 extracts static `pack`/`block`/`sky` helpers. `unpack` is the
  same split in both. The Rust's `text_display.rs:269-277` already does
  exactly the 1.21.4 form. `Display$TextDisplay` save/load differ only
  by codec form: 1.21.4 saves `text` as a JSON string and reads
  `line_width`/`background`/`text_opacity` behind `contains(key, 99)`
  guards with defaults 200 / `0x40000000` / -1; the Rust's
  `text_display_data` already matches those defaults (flags shadow 1 /
  see-through 2 / default-background 4 / align left 8 / right 16).
  Rank 2: 167/348.

  Item 122 (horizon-client-render/signs.rs): PORTED.
  `SignBlockEntity.executeClickCommandsIfPresent` and
  `SignText.hasAnyClickCommands` differ only by the `ClickEvent`
  representation: 1.21.5 pattern-matches the `RunCommand` record and
  reads `command()`; 1.21.4 checks `getAction() == RUN_COMMAND` and
  reads `getValue()`. The Rust's `signs.rs::click_action` read
  `event.get("command")` -- the 1.21.5 form -- so a waxed sign whose
  `run_command` click event came from vanilla 1.21.4 (command in `value`)
  never fired. Fix: `click_action` now reads `value` first, falling back
  to `command`, with the doc comment corrected to `clickEvent`. Test
  added: `a_run_command_click_reads_its_command_from_value`. Rank 2:
  168/348.

  Item 123 (horizon-client/health.rs): PORTED.
  `SweetBerryBushBlock.useWithoutItem` in 1.21.4: if `age > 1`, pop
  `1 + nextInt(2) + (age == 3 ? 1 : 0)` sweet berries, play
  `SWEET_BERRY_BUSH_PICK_BERRIES` (volume 1.0, pitch
  `0.8 + nextFloat()*0.4`), set age to 1 (flags 2), emit `BLOCK_CHANGE`,
  return `SUCCESS`. There is no `harvest/sweet_berry_bush` loot table in
  1.21.4 (the table arrived in 1.21.9); the Rust instead emitted
  `PlayerEffect::InteractLoot("minecraft:harvest/sweet_berry_bush")`,
  which resolves to nothing when the table is absent, so right-clicking
  a mature bush dropped no berries -- the exact defect reported live in
  1.21.7 on Nov 25, 2026. Fix: new `PlayerEffect::DropStack(Arc<str>,
  u32)` variant; the bush arm draws the count first (before the pitch
  draw, as vanilla draws it) and emits `DropStack`; `game.rs` handles
  it via `pop_resource` (widened to `pub(super)`), and the server-side
  `LevelEvent::Player` arm treats it as already popped. Test added:
  `picking_a_grown_sweet_berry_bush_drops_berries_directly`. The other
  123 classes match: `entityInside` (stuck multiplier `(0.8, 0.75,
  0.8)`, foxes/bees exempt, age 0 exempt, prick 1.0 at `|x| >= 0.003` or
  `|z| >= 0.003`), `performBonemeal` (`min(3, age+1)`, flags 2),
  `DamageEffects` identical. Recorded, NOT fixed (needs their own
  focused items): the cave-vine arm in the same `interact.rs` still
  uses `InteractLoot("minecraft:harvest/cave_vine")`, and the
  beehive-shear path still uses a harvest table -- both tables are
  absent in 1.21.4 and both need the same direct-drop treatment. Rank 2:
  169/348.

  Item 124 (horizon-client/examples/probe/scene_motion_probe.rs):
  PORTED. `NetherPortalBlock.randomTick` in 1.21.4: natural dimension,
  `doMobSpawning`, `nextInt(2000) < difficulty.getId()`, walk down while
  the block is a portal, `isValidSpawn` for `ZOMBIFIED_PIGLIN` at the
  block below, spawn with `EntitySpawnReason.STRUCTURE` at
  `below.above()`, `setPortalCooldown()` on the piglin and its vehicle.
  There is NO `anyPlayerCloseEnoughForSpawning` check in 1.21.4's
  `randomTick`; 1.21.5 added it. The Rust's `portal_spawn` applied the
  1.21.5 gate (non-spectator within 128 blocks of the chunk centre) and
  returned early without it. Fix: the gate is removed. The tick side
  already matches (natural dimension, `spawn_mobs` + `spawn_monsters`
  reducing to `doMobSpawning`, difficulty roll, `portal_spawn` event);
  the spawn itself matches (walk down, sturdy top face, spawn at
  `+0.5 / y+1 / +0.5`, cooldown; no vehicle is ever attached by a
  zombified piglin's `finalizeSpawn`). `getEntityInsideCollisionShape`:
  1.21.5's portal override returns `getShape`, 1.21.4 has no override
  (base form is the whole block) -- the Rust's `inside_blocks` already
  tests whole-block overlap, the 1.21.4 form. Rank 2: 170/348.

  Item 125 (horizon-client/game/block_items.rs): PORTED.
  `BambooSaplingBlock.getDestroyProgress` exists ONLY in 1.21.4: a
  `SwordItem` in the main hand returns 1.0 (instant break), else it
  delegates to `Block.getDestroyProgress`. 1.21.5 dropped the override.
  The Rust's `destroy_progress` had no per-block override, so the
  sapling mined at the ordinary rate (hardness 1.0: `fconst_1` then
  `strength(1.0F)` in `Blocks`, verified in the 1.21.4 jar). Fix:
  `destroy_progress` returns 1.0 for `minecraft:bamboo_sapling` when
  the held item is in the `minecraft:swords` tag (the tag exists in
  1.21.4 with all six swords), placed before the hardness check as the
  override is before its super call. Test added:
  `a_sword_breaks_a_bamboo_sapling_at_once` (#[ignore], needs prepared
  resources). Incidental item-119 follow-up found while fixing: the
  first-person `destroy_block` in the same file still had the 1.21.5
  `Item.canDestroyBlock` guard (creative + sword/trident/mace -> cannot
  break), which item 119 removed from `game.rs`, `remote.rs` and
  `guide.rs` but missed here; removed, with the
  `first_person_probe.rs` mock fixed the same way. The stems carry
  over: shape construction is prepare-derived, `VegetationBlock` is a
  pure rename of `BushBlock` (identical method bodies, verified), and
  `randomTick`/`performBonemeal`/`updateShape` match.
  `BaseContainerBlockEntity` `CustomName`: 1.21.4 uses the JSON-string
  form, 1.21.5 the NBT codec form -- the known item-163 gap; the
  Rust's loader already accepts both forms. Rank 2: 171/348.

  Item 126 (horizon-client/mobs/living.rs): PORTED.
  `Goat.calculateFallDamage` is `super - 10` in both versions;
  `Breeze.causeFallDamage` plays `BREEZE_LAND` over 3.0 blocks in
  both; both were already implemented. The real delta is the BASE
  formula: 1.21.4 `LivingEntity.calculateFallDamage(float, float)`
  returns `Mth.ceil((fallDistance - safe) * multiplier *
  fallDamageMultiplier)` with the subtraction and first multiplication
  in float; 1.21.5 moved this to `calculateFallPower(double) =
  fallDistance + 1.0E-6 - safe` and floors the product. The Rust used
  the 1.21.5 form. Fix: `cause_fall_damage_at` now computes `power =
  (distance as f32 - safe as f32) * modifier` and `damage =
  ceil(power as f64 * fallDamageMultiplier)`; the goat -10 / frog -5
  still apply after it. Test added:
  `a_half_block_fall_costs_the_ceiled_damage` (a 4.5-block fall onto
  stone costs a pig ceil(1.5) = 2, health 10 -> 8; runs in CI, returns
  early without prepared resources locally). The remaining
  float-vs-double precision is the item-101 tree-wide question. Rank 2:
  172/348.

  Item 127 (horizon-world/loot.rs): carry-over.
  `EntityPredicate.matches` is identical except `fallDistance` is
  float in 1.21.4 vs double in 1.21.5 (item 101's f64 question); the
  CODEC gains a 16th `components` field in 1.21.5. The Rust's
  `entity_matches` does not evaluate `movement`/`distance`/`location`/
  `effects`/`nbt`/`slots` at all -- a pre-existing loot-engine gap
  identical in both versions, recorded. `ToggleTooltips`: 1.21.5
  writes ids into `tooltip_display.hidden_components`; 1.21.4 mutates
  each named component's own `show_in_tooltip` flag in place. The
  Rust's loot function emits the tree's canonical 1.21.5 form, which
  the tooltip layer reads; the 1.21.4 form is converted on load by
  datafix 4307. There is no reverse conversion on save/wire (the
  v1_21_4 protocol adapter has no tooltip handling) -- recorded as a
  boundary gap in the same class as item 113's protocol-side
  `DataComponentPredicate` note. `SetInstrumentFunction`: the Rust
  already writes the 1.21.4 bare-id form. `SetWrittenBookPagesFunction`:
  `list_operation` defaults a missing mode to `replace_all`, so a
  1.21.4 plain pages list replaces all pages. `LootData` is
  datagen-only. Rank 2: 173/348.

  Item 128 (horizon-world/worldgen/structure/legacy.rs): carry-over.
  All diffs are codec-form reshuffles with identical NBT and results:
  `createTag` (`store("BB", CODEC)` vs `encodeStart` +
  `resultOrPartial` + `ifPresent`), the tag ctor (`getIntOr` vs
  `getInt`, `read(...).orElseThrow()` vs `parse(...).getOrThrow()`),
  the template ctor (`getStringOr`/`getIntOr` vs `getString`/`getInt`),
  `postProcess` (`StructureMode` via `LEGACY_CODEC` vs `getString` +
  `valueOf`). The Rust's `piece_tag` writes exactly 1.21.4's fields
  (`id`, `BB` int array of 6, `O`, `GD`), and the readers match the
  1.21.4 forms. Rank 2: 174/348.

  Item 129 (crates/horizon-client/src/block_entity_item.rs):
  carry-over. `Spawner.appendHoverText` / `getSpawnEntityDisplayName`
  differ only by call shape (1.21.4 takes the `ItemStack` and reads the
  `BLOCK_ENTITY_DATA` component; 1.21.5 takes the `CustomData`
  directly); the logic is identical and the Rust already implements the
  1.21.4 form (`spawner_entity` + the desc1/desc2 fallback in
  `game/stations.rs`). Noted edge case, not fixed: the Rust does not
  validate the id against the entity registry, so an unregistered or
  unparseable id would show as a raw line where vanilla falls back to
  desc1/desc2 -- only reachable with corrupt/modded data. Rank 2:
  175/348.

  Item 130 (crates/horizon-client/src/creative.rs): carry-over.
  `hasInfiniteMaterials` (1.21.5) vs `hasInfiniteItems` (1.21.4) is the
  same creative-mode condition. The 1.21.5 early return for empty
  tooltip lines in `getTooltipFromContainerItem` is a no-op in
  practice; the Rust's `creative_tooltip_tabs` implements the 1.21.4
  tab-name/tag annotation. The spawn-eggs tab icon differs
  (`CREEPER_SPAWN_EGG` vs `PIG_SPAWN_EGG`) but tab icons are
  data-driven from the version's own prepared export, so 1.21.4
  carries the pig egg by construction. Menu chrome per the owner's
  principle. Rank 2: 176/348.

  Item 131 (crates/horizon-client/src/game/clone_block_ticks_tests.rs):
  PORTED. 1.21.5 added `strict` to `/clone`; 1.21.4 has none. Removed
  the `strict` literal from the command tree
  (`horizon-command/src/vanilla.rs`, `clone_destination`) and the
  `ctx.has("strict")` read in `clone_between` (`game/commands.rs`):
  destinations now place via `place_state(..., false, false)`
  (`level_set`, flags 3, neighbours notified inline -- the same end
  state as vanilla's flags 2 + `blockUpdated` pass, only the
  notification ordering differs, and that ordering is a pre-existing
  approximation, identical in both versions). The move clear was
  using the strict 818 flags; vanilla 1.21.4 clears with
  `setBlock(pos, AIR, 3)`, so it is now `place_state(from, air, false,
  false)`. Tests updated: the six clone commands in
  `clone_block_ticks_tests.rs` drop `strict` (first test renamed
  `a_clone_copies_the_pending_block_ticks_of_its_source`; assertions
  still hold -- the placed sand's own fall tick has the same trigger
  as the source's, no tick advancing between the commands) and
  `horizon-command/src/tests.rs` now asserts the strict form fails to
  parse. `LevelChunkTicks` save/load: the Rust's `tick_data.rs`
  already writes the 1.21.4 `SavedTick` NBT form (`i, x, y, z, t, p`);
  `copyAreaFrom` is called by both versions and by the Rust. Rank 2:
  177/348.

  Item 132 (crates/horizon-client/src/game/storage.rs): carry-over.
  `ShulkerBoxBlockEntity.loadFromTag`'s extra `contains("Items", 9)`
  guard is a no-op; the 1.21.5-only `preRemoveSideEffects` override
  is an empty return suppressing the base contents-drop, which the
  Rust already models (`removal::pre_remove` returns `ShulkerBox`:
  keeps its contents for the dropped box). `startOpen`/`stopOpen`
  and the barrel's `playSound` differ only by the `Level.playSound`
  overload (Entity vs Player); volume 0.5, pitch
  `nextFloat()*0.1+0.9`, and the barrel's `pos + 0.5 + facing/2`
  position are exactly what the Rust's `container_sound` does.
  `PlayerEnderChestContainer.fromTag`'s `getCompoundOrEmpty` /
  `getByteOr` reshuffle matches the Rust's `decode_items` (missing
  Slot defaults to 0, out-of-range slots skipped). Rank 2: 178/348.

  Item 133 (crates/horizon-client/src/mobs/creaking.rs): PORTED.
  `Creaking.hurtServer` in 1.21.5 emits `gameEvent(ENTITY_ACTION)`
  after `broadcastEntityEvent(66)`; 1.21.4 goes straight to
  `level.getBlockEntity(home)`, so the game-event call in the Rust
  hurtServer echo was removed (the aiStep/can-move `ENTITY_ACTION`
  exists in both versions and stays; `activate`/`deactivate` still
  post it, pinned by `activating_and_deactivating_post_entity_action`).
  `Creaking.canBeNameTagged` exists ONLY in 1.21.4 (heart-bound ->
  false); 1.21.5 dropped the check from
  `NameTagItem.interactLivingEntity`, so a heart-bound creaking takes
  a name tag there but not in 1.21.4. Fixed: `game/names.rs`
  `name_mob` refuses a heart-bound creaking, and
  `game/remote_mob_use.rs` client prediction no longer claims the
  named tag for one. New tests
  `game/creaking_name_tag_tests.rs` (bound refuses, free takes).
  Recorded, not fixed: the 1.21.5 Brain additions
  (`NEAREST_VISIBLE_ATTACKABLE_PLAYERS` memory +
  `isAttackTargetStillReachable` stop condition, FIGHT registered
  via `addActivityWithConditions` instead of
  `addActivityAndRemoveMemoryWhenStopped`) -- the Rust creaking AI
  is a custom non-Brain implementation, and adding the gate would ADD
  1.21.5 behaviour; 1.21.4 behaviour is approximated by the generic
  melee + activate/deactivate. Also the `tearDown` creaking-heart
  property form (`STATE = AWAKE` enum vs `ACTIVE = true` boolean) --
  the tree's internal block-state form is the 1.21.5 one (datafix
  4294 renames `active` -> `creaking_heart_state` in saved data; the
  wire adapter maps down), a tree-wide/registry-form question out of
  scope for this item. Rank 2: 179/348.

  Item 134 (crates/horizon-client/src/mobs/illager.rs): carry-over.
  `EvokerWololoSpellGoal` differs only by the `Sheep` package move
  (`animal/sheep/Sheep` vs `animal/Sheep`); the wololo target must
  be a blue sheep and the cast sets it red, which the Rust's
  `illager.rs` already does (`color == 11`, `MobEvent::Wololo`).
  `EvokerSummonSpellGoal.performSpellCasting` differs only by
  `Vex.snapTo` vs `Vex.moveTo` (the 1.21.5 rename of the
  position+rotation setter); the Rust's `summon_vexes` matches
  (three vexes at base + (-2..+2, +1, -2..+2), life
  20*(30+nextInt(90)), `ENTITY_PLACE` game event after each).
  Rank 2: 180/348.

  Item 135 (crates/horizon-world/src/datafix/dynamic.rs): carry-over.
  The `Tag.as*()` accessors (`asNumber`, `asByte`, `asString`,
  `asCompound`, `asList`, ...) are a 1.21.5 API addition; 1.21.4 has
  `getAsString()` instead. Their semantics match 1.21.4's
  `NbtOps.getNumberValue` (numeric tags only), verified by javap on
  both jars, and the Rust's `dynamic.rs` helpers already read numbers
  the 1.21.4 way. Noted corner, not fixed: 1.21.5 `Tag.asBoolean`
  maps through `asByte` (byte != 0), so a fractional Float 0.5 reads
  false, while the Rust's `as_bool_opt` uses `double_value() !=
  0.0`; no datafix reads a boolean from a float (every `as_bool`
  caller reads Byte-valued flags), so no real data diverges.
  Rank 2: 181/348.

  Item 136 (crates/horizon-world/src/level/growth.rs): PORTED.
  `SaplingBlock.advanceTree` cycles the stage with flags 4 in
  1.21.4 (`UPDATE_INVISIBLE`) and 260 in 1.21.5 (`UPDATE_NONE`);
  `TreeGrower.growTree` uses flags 4 for all ten of its `setBlock`
  calls in 1.21.4 (260 in 1.21.5). The Rust used `UPDATE_NONE`
  (260) for the stage cycle and both restores, and 818
  (`UPDATE_SKIP_ALL_SIDEEFFECTS | UPDATE_CLIENTS`) for
  `remove_sapling` -- 818 sends client updates and skips neighbour
  shape updates, which 1.21.4 does not. All four sites now use
  `flags::UPDATE_INVISIBLE`. The mega path sets AIR directly in
  vanilla while the Rust sets the fluid's legacy block -- the same
  here because a sapling is never waterlogged. 1.21.4's `growTree`
  also calls `sendBlockUpdated(pos, getBlockState(pos), legacyBlock,
  2)` after a successful single-tree placement when the spot is
  still the legacy block -- not added: the world's `set()` logs
  every real change into `block_log` and the game broadcasts it at
  tick end, so the removal already reaches clients in this
  architecture. Sapling/mushroom shapes are prepare-derived and
  geometrically identical anyway (`column(12,0,12)` ==
  `box(2,0,2,14,12,14)`; `column(6,0,6)` ==
  `box(5,0,5,11,6,11)`); `MushroomBlock.randomTick` differs by
  whitespace only. Rank 2: 182/348.

  Item 137 (client/renderer/texture): carry-over. Widely named: 8
  classes, 26 methods. The whole delta is the 1.21.5 Blaze3D
  GPU-texture migration (`getId`/`bind`/`releaseId` + GL enums ->
  `GpuTexture`/`AddressMode`/`createCommandEncoder`; `upload` /
  `tickAndUpload` gain a `GpuTexture` parameter). The Rust renderer
  is wgpu-based and never used vanilla's GL ids. `OverlayTexture`'s
  content is identical in both versions (16x16, hurt texel
  -1291911168 = 0xB2FF0000, white-row alpha
  `(1-u/15*0.75)*255`); the Rust's `overlay.rs` implements exactly
  that, JVM-verified. `InterpolationData` has no diffs at all.
  Rank 2: 183/348.

  Item 138 (crates/horizon-client-render/src/biome.rs): PORTED.
  `Biome.getGrassColor` is identical in both versions modulo the
  factor-out (base = override or `getGrassColorFromTexture`, then
  the modifier); `FoliageColor.get` is a pure factor-out
  (1.21.5's `ColorMapColorUtil.get` is byte-for-byte the 1.21.4
  inline computation, fallback -12012264 = 0x48B518); the Rust's
  `sample()` matches, and its grass fallback 0xFF00FF matches
  1.21.4's -65281. The real delta: 1.21.5 added
  `getDryFoliageColor` / a `dryFoliageColorOverride` field, and
  1.21.4 ships no `dry_foliage.png` colormap -- but `Biomes::load`
  required it (`colormap(root, "dry_foliage")?`). PROVEN with a
  minimal 1.21.4 root (grass+foliage maps + plains.json):
  `Biomes::load` FAILED with `No such file or directory`, so the
  1.21.4 game could not load biomes (`BlockStates::load` unwraps
  it). This overturns item 115's premise that the kept dry-foliage
  machinery is harmless data-driven weight. Fix: the dry map load
  is now tolerated (`unwrap_or_default`); an empty map samples to
  the 0x5D4930 fallback, and no 1.21.4 biome or block reads it.
  Regression test
  `load_succeeds_without_a_dry_foliage_colormap` (root with
  grass/foliage only) passes; biome lib tests 8 passed, mesher 65
  passed locally. The `dry_foliage` field, `Tint::DryFoliage` /
  `Resolver::Dry` and the Polytone `dry_foliage` function stay,
  per item 115 -- only the hard load requirement is gone.
  Rank 2: 184/348.

  Item 139 (crates/horizon-client/src/mobs/guardian.rs): carry-over.
  Elder sounds: 1.21.5 checks `isInWater()`, 1.21.4
  `isInWaterOrBubble()`; the Rust's fluid table maps
  `bubble_column` to WATER, so `in_water` is already true in a
  bubble column and the Rust's elder-sound checks already behave
  as 1.21.4 (vanilla's `BubbleColumnBlock.getFluidState` is water
  too, so the 1.21.5 change is unobservable in practice).
  `customServerAiStep`: `MINING_FATIGUE` vs `DIG_SLOWDOWN` is a
  field rename -- the registry id is `minecraft:mining_fatigue`
  in both, and the Rust gives it 6000/2 with the same 50-block
  range and `endsWithin(1199)` guards. `GuardianRenderer.renderBeam`
  differs only by `mulPose(Quaternionfc)` vs `mulPose(Quaternionf)`.
  Rank 2: 185/348.

  Item 140 (crates/horizon-client/src/mobs/trial_spawner.rs):
  carry-over. All diffs are API renames with identical semantics:
  `playSound` overload Entity vs Player (null first argument either
  way); `WeightedList.getRandom` vs
  `SimpleWeightedRandomList.getRandomValue`/`getRandom` for the
  loot tables, spawn potentials and dispensing items; `snapTo` vs
  `moveTo`; `lambda$tickAndGetNext$0` takes the `SpawnData`
  directly in 1.21.5 vs unwrapping `WeightedEntry$Wrapper.data()`
  in 1.21.4, same stored `nextSpawnData` + `markUpdated`. The Rust's
  `tick_and_get_next` matches the state machine and
  `spawn_ominous_item_spawner` matches the ominous-item lambda
  (pitch `(nextFloat-nextFloat)*0.2+1.0`, `spawn_item_begin`,
  `cooldownEndsAt = gameTime + ticksBetweenItemSpawners`).
  Rank 2: 186/348.

  Item 141 (crates/horizon-client/src/ticket_levels.rs): carry-over.
  `Ticket` was reworked in 1.21.5: 1.21.4's `Ticket<T>` has a key,
  `createdTick` and `timedOut(now)` = `timeout != 0 && (now -
  createdTick) > timeout`; 1.21.5's has `ticksLeft`,
  `decreaseTicksLeft` / `resetTicksLeft` / `isTimedOut` and a CODEC
  (`TicketStorage` persists portal tickets to
  `chunk_tickets.dat`). PORTAL timeout is 300 in both versions. The
  Rust's countdown (`ticks_left` = 300, decrement each tick, retain
  while >= 0) survives exactly 301 ticks, the same as vanilla
  1.21.4 (alive at C+300, out at C+301); re-adding a same
  chunk+level ticket restarts the timeout in both. 1.21.4 has no
  `TicketStorage`: the Rust's `load_portal_tickets` returns an empty
  vec and documents that portal tickets are not persisted in 1.21.4.
  Rank 2: 187/348.

  Item 142 (world/entity/npc): carry-over. `VillagerData` is a
  record in 1.21.5 (Holder type/profession, `withLevel` /
  `withType` / `withProfession`) vs a mutable class in 1.21.4
  (plain values, get/set, codec via `byNameCodec` not
  `holderByNameCodec`) -- API shape; the Rust models villager data
  as mob variants. Behaviour matches 1.21.4: NBT `{type,
  profession, level}` with level clamped 1..5, default 1; XP
  thresholds `[0, 10, 70, 150, 250]`; a new profession clears the
  offers. `AbstractVillager` diffs are whitespace only.
  Rank 2: 188/348.

  Item 143 (crates/horizon-client-render/src/chunk.rs): carry-over.
  `SerializableChunkData.write` differs only by codec form
  (`store`/`storeNullable` vs spelled-out `encodeStart` +
  `resultOrPartial` + `ifPresent`) -- same NBT out, and the Rust's
  `save.rs` `encode_with` writes the 1.21.4 chunk NBT (DataVersion,
  xPos/zPos/yPos, Status full, LastUpdate, isLightOn, sections,
  block_entities, InhabitedTime, structures References + starts,
  PostProcessing, Heightmaps, block/fluid ticks), preserving
  blending_data / below_zero_retrogen / UpgradeData from the loaded
  extras. Noted corners, not fixed (identical in both versions):
  the Rust's References read keeps unknown-structure ids and
  unfiltered positions where vanilla warns-and-discards unknown
  structures and filters by `isInside`. Rank 2: 189/348.

  Item 144 (crates/horizon-client-render/src/particles.rs):
  carry-over. `WeightedRandom`: 1.21.5 generalized the pick to a
  weight function; the algorithm is identical, and the Rust's
  `spawner::weighted_index` matches the 1.21.4 walk (subtract
  weights, first index where the remainder < 0). `ItemStackRenderState`:
  the Rust has no render-state struct; its breaking-item particle
  picks a random layer's sprite, matching 1.21.4's
  `pickParticleIcon` (random layer -> model's particle icon, null
  when the model is null). The 1.21.4-only `isGui3d()` /
  `transform()` and the `clear()` `isLeftHand` reset, and the
  1.21.5-only `visitExtents`, are API details of a struct the Rust
  does not express. Rank 2: 190/348.

  Item 145 (crates/horizon-client/src/game/decorated_pot.rs):
  carry-over. `DecoratedPotBlockEntity.loadAdditional` /
  `saveAdditional` differ by codec form only: 1.21.5 reads the
  sherds via `CompoundTag.read("sherds", PotDecorations.CODEC)`
  (default EMPTY) and the item via `ItemStack.CODEC` with a
  `RegistryOps` context; 1.21.4 uses `PotDecorations.load(tag)`
  (EMPTY when absent or when there is no `sherds` key) and
  `ItemStack.parse(provider, tag.getCompound("item"))` behind a
  `contains("item", 10)` guard, falling back to EMPTY when a loot
  table was loaded. Save mirrors it: 1.21.5 stores via
  `PotDecorations.CODEC` only when not EMPTY and the item via
  `ItemStack.CODEC`; 1.21.4 calls `PotDecorations.save(tag)`
  (unchanged when EMPTY) and `ItemStack.save(provider)` when the
  item is non-empty and no loot table is set.
  `PotDecorations.load` / `save` use the same CODEC under the hood
  in 1.21.4, and `splitTheItem` differs by whitespace only. The
  Rust's `block_entity_nbt.rs` decorated-pot arm already matches
  the 1.21.4 form (four sides as item templates, `item` loaded
  only when there is no loot table), covered by
  `decorated_pot_string_template_load_save_uses_item_compound`.
  Rank 2: 191/348.

  Item 146 (crates/horizon-world/src/level/pistons.rs): PORTED.
  `PistonBaseBlock.moveBlocks` head-clear: 1.21.5 clears a held
  piston head with `setBlock(pos, AIR, 276)`; 1.21.4 uses flags
  20. The Rust used 276 — now 20. The two moving-piston
  placements use 324 in 1.21.5 and 68 in 1.21.4 (the 256
  `SKIP_BLOCK_ENTITY_SIDEEFFECTS` bit is the 1.21.5 addition);
  the Rust used 324 — now 68. `triggerEvent`'s retract
  moving-piston set is 276 in 1.21.5, 20 in 1.21.4; the Rust
  used 276 — now 20. The destroy loop differs in kind, not just
  flags: 1.21.5 sends `levelEvent(2001)` (particles plus the
  break sound) when the level is client-side and the state is not
  `#minecraft:fire`, before clearing the block; 1.21.4 sets AIR
  (18), posts `BLOCK_DESTROY`, then calls `addDestroyBlockEffect`
  (particles only, no sound). Verified by javap on both jars:
  `Level.addDestroyBlockEffect` is a no-op on the server and
  `ClientLevel` overrides it to `ParticleEngine.destroy`, whose
  only caller is `moveBlocks` — so on the server the effect is
  nothing, and the 2001 event is never sent by a 1.21.4 piston.
  The Rust had the 1.21.5 form, so the loop now uses the 1.21.4
  order and emits a new `LevelEvent::DestroyParticles` (particles
  only) for a non-fire state on the client copy;
  `game.rs::process_level_events` plays it with no break sound,
  and `block_update_fixture.rs` has a documented no-op arm (its
  probe level is server-side, so the event is never emitted
  there). `checkIfExtend`, `mirror` and `rotate` differ by branch
  offsets and `checkcast` spacing only; `getShape` and the static
  init build the same per-direction extended shapes (1.21.5 via
  `Shapes.rotateAll`, 1.21.4 via six AABBs) — the Rust's shapes
  are prepare-derived. Rank 2: 192/348.

  Item 147 (widely named: server/level): carry-over.
  `TicketType` was reworked in 1.21.5: `register(name, timeout,
  boolean, TicketUse)` plus `doesLoad` / `doesSimulate` /
  `hasTimeout` / `persist`, final `equals` / `hashCode`, and the
  `TicketUse` enum (LOADING / SIMULATION); 1.21.4's constructor
  is `TicketType(name, Comparator, timeout)`. The timeouts are
  identical in both versions (PORTAL 300, ENDER_PEARL 40, UNKNOWN
  1; START / DRAGON / PLAYER / FORCED have none; 1.21.5 splits
  PLAYER into `player_loading` / `player_simulation`). The Rust
  models only PORTAL tickets (300-tick countdown, verified in
  item 141) and chunk loading by view distance, so the rework is
  API shape with no behaviour to port. `ServerPlayer$1`
  (ContainerSynchronizer): `broadcastDataValue`,
  `sendCarriedChange` and `sendSlotChange` are identical modulo
  javap spacing; `sendInitialData` is `List` vs `NonNullList`
  with the same content (slots + carried + data values) — the
  Rust's `lan/containers.rs::send_menu_content` matches (menu
  slots + 36 player slots + carried, state id incremented);
  `createSlot` / RemoteSlot is 1.21.5-only and the Rust has no
  remote-slot concept, correct for 1.21.4.
  `DistanceManager$FixedPlayerDistanceChunkTracker.runAllUpdates`
  differs by spacing only; `dumpChunks` is a 1.21.4-only debug
  method. `ServerPlayer$1$1` and `TicketType$TicketUse` are
  1.21.5-only. Rank 2: 193/348.

  Item 148 (crates/horizon-client-render/src/audio.rs):
  carry-over. `com/mojang/blaze3d/audio/Library.init` differs
  only by a factor-out: 1.21.4 builds the attributes buffer
  inline (`callocInt(3)`, putting 6554 for HRTF) and calls
  `setHrtf(boolean)`; 1.21.5 factors the buffer into
  `createAttributes(stack, boolean)`. Same behaviour: HRTF is
  enabled only when the device reports `ALC_SOFT_HRTF` and the
  option is on. `getChannelCount` and `getDefaultDeviceName`
  differ by `ldc` vs `ldc_w` and branch offsets only. The delta
  lives entirely inside LWJGL / OpenAL device init, which the
  Rust does not use: `audio.rs` is the Rust's own mixer standing
  in for OpenAL, with no HRTF or ALC device enumeration to port.
  Rank 2: 194/348.

  Item 149 (crates/horizon-client-render/src/gfx/pack_atlas.rs):
  carry-over. `SpriteLoader.stitch` differs only by 1.21.5
  wrapping the body in a profiler zone (`Profiler.get().zone`,
  try-with-resources — the extra exception-table entries and
  slot renumbering); the stitch logic (Stitcher init, the min
  width / height across sprites, `lowestOneBit`, the mip level)
  is identical. `SectionCompiler.compile` differs by branch
  offsets only. The Rust's `pack_atlas.rs::stitch_mip_level`
  matches the shared computation (smallest = min side, lowest =
  min lowestOneBit via `side & side.wrapping_neg()`, level =
  log2(min(smallest, lowest)) clamped to the option). The
  lambda renumbering (`loadAndStitch` / `runSpriteSuppliers` /
  `stitch`) is the same bodies under new numbers. Rank 2:
  195/348.

  Item 150 (crates/horizon-client/src/game/nether_portal.rs):
  carry-over. `PositionMoveRotation.calculateDelta` /
  `calculateAbsolute` / `of` show no substantive differences
  (spacing and branch offsets only). The real shape difference
  is that 1.21.5's `of` reads the `InterpolationHandler`'s
  position / yaw / pitch when the entity is interpolating, while
  1.21.4's `of` uses `position()` + `getKnownMovement` + yaw /
  pitch; 1.21.4 also has a separate `ofEntityUsingLerpTarget`
  (lerp target + known movement + yaw / pitch), which has no
  callers in the likely classes and is a 1.21.4-only helper. The
  Rust's `nether_portal.rs` uses the 1.21.4 forms:
  `end_portal_rotation` implements `calculateAbsolute` for End
  portal transitions (west-facing into the End, spawn angles
  out, pitch clamped to +/-90), and the minecart path cites
  `PositionMoveRotation.of(this)` taking `getKnownMovement`
  (clamped to 0.4). No `InterpolationHandler` exists in 1.21.4,
  so there is nothing to port. Rank 2: 196/348.

  Item 151 (crates/horizon-client/src/game/sculk.rs):
  carry-over. `SculkShriekerBlockEntity.tryShriek` /
  `tryRespond` / `trySummonWarden` / `playWardenReplySound`
  and `loadAdditional` / `saveAdditional` show no substantive
  differences — the DIFFERS flags come from slot renumbering
  and the codec / lambda reshuffle (Supplier vs Consumer
  invokedynamic), same NBT in and out.
  `SculkShriekerBlock.getCollisionShape` /
  `getOcclusionShape` differ only by the `SHAPE_COLLISION` ->
  `COLLIDER` field rename; `stepOn` the same rename. The
  mechanism moved, not the behaviour: 1.21.4's
  `SculkShriekerBlock.onRemove` calls `tryRespond` when the
  block is shrieking and the new state is a different block
  (server-side); 1.21.5's
  `SculkShriekerBlockEntity.preRemoveSideEffects` calls
  `tryRespond` when `SHRIEKING` on a `ServerLevel`. The Rust's
  `sculk.rs::sculk_sync_writes` already matches both: a
  same-kind change is skipped, then a shrieking shrieker whose
  entity exists responds (`shrieker_respond`: warning > 0,
  summon at 4 else the warning-level reply sound, darkness 40).
  One comment corrected to cite the 1.21.4 `onRemove` form
  instead of the 1.21.5 `preRemoveSideEffects` name. Rank 2:
  197/348.

  Item 152 (crates/horizon-client/src/game/spawner.rs): PORTED.
  `TrialSpawner.spawnMob`: 1.21.5 reads `Pos` through
  `Vec3.CODEC` with a computed-position fallback, so a list
  shorter than three rolls all three components; 1.21.4 reads
  the `Pos` list component by component (`getList("Pos", 6)`;
  size >= 1 -> x from list[0], else `pos.x +
  (nextDouble() - nextDouble()) * spawnRange + 0.5`; size >= 2
  -> y from list[1], else `pos.y + nextInt(3) - 1`; size >= 3
  -> z from list[2], else the same roll as x). Verified
  1.21.4's `BaseSpawner.serverTick` uses the identical
  per-component read. The Rust's `spawner::read_pos` required
  exactly three entries (the 1.21.5 codec form) and was used by
  both the base spawner and the trial spawner. Fixed with a new
  `spawner::spawn_pos(tag, pos, range, random)` implementing
  the 1.21.4 per-component form (random draws only for missing
  components, in x, y, z order) and both call sites now use it.
  New test `spawn_pos_reads_each_component_on_its_own` (full
  list as-is including integer numerics; a one-component list
  keeps x and rolls y then z; no list rolls all three) with
  mirror-random expectations; runs via CI (horizon-client lib
  tests OOM on the VM). `SpawnerRenderer` /
  `TrialSpawnerRenderer` render overloads differ only by the
  1.21.5 `Vec3` camera parameter (API);
  `renderEntityInSpawner` is identical. Rank 2: 198/348.

  Item 153 (crates/horizon-client/src/lan/containers.rs):
  carry-over. `AbstractContainerMenu`'s diffs are the 1.21.5
  RemoteSlot / HashedStack sync rework:
  `RemoteSlot.PLACEHOLDER` in the constructor / `addSlot`,
  `ContainerSynchronizer.createSlot`,
  `RemoteSlot.force` / `matches` in
  `synchronizeCarriedToRemote` / `synchronizeSlotToRemote` /
  `sendAllDataToRemote` / `setRemoteSlot` / `transferState`,
  `setRemoteCarried(HashedStack)` / `setRemoteSlotUnsafe` vs
  `setRemoteCarried(ItemStack)` / `setRemoteSlotNoCopy`.
  `doClick`, `broadcastChanges`, `broadcastFullState`,
  `setData`, `addDataSlot`, `stillValid` and `removed` are
  identical modulo spacing and lambda renumbering. The Rust has
  no RemoteSlot / HashedStack concept: `lan/containers.rs`
  sends `ContainerSetContent` / `ContainerSetSlot` with whole
  stacks (the 1.21.4 form) and `claimed_matches` already
  documents "1.21.4 sends the whole stack, 1.21.5 a hash of
  it". Nothing to port. Rank 2: 199/348.

  Item 154 (crates/horizon-client/src/mobs/aquatic.rs):
  carry-over. `Squid.aiStep`, `Squid.travel` and
  `AgeableWaterCreature.handleAirSupply` show no substantive
  differences — no changed calls, fields or constants, only
  slot numbers and branch offsets. The Rust's `aquatic.rs`
  inherits the shared logic. Rank 2: 200/348.

  Item 155 (crates/horizon-client/src/mobs/vex.rs): PORTED
  (re-verification of a batch-3 closure). `Vex`
  `addAdditionalSaveData` / `readAdditionalSaveData`: 1.21.5
  stores the bound origin as `bound_pos` via `BlockPos.CODEC`
  (int array) and the life as `life_ticks`; 1.21.4 writes
  `BoundX` / `BoundY` / `BoundZ` ints (only when `boundOrigin`
  != null) and `LifeTicks` (only when `hasLimitedLife`). This
  item was first closed in batch 3 as ported, but
  re-verification in batch 44 found `vex.rs` still wrote the
  1.21.5 form for both keys, so a 1.21.4 vanilla save's bound
  origin and life were lost on load and a Rust-written save
  was unreadable by vanilla 1.21.4. Fixed in batch 44:
  `write_nbt` now writes `BoundX` / `BoundY` / `BoundZ` when
  home is set and `LifeTicks` when `egg_time` > 0 (equivalent
  to `hasLimitedLife` — a limited-life vex never has
  `egg_time` 0 at save time); `read_nbt` reads `BoundX` / `Y` /
  `Z` when `BoundX` is present and `LifeTicks`, -1 (unlimited)
  when absent. New test
  `a_bound_vex_saves_bound_xyz_and_life_ticks` (keys present, no
  1.21.5 keys, round-trip restores home and `egg_time`; a free
  vex writes neither key). `Vex.tick` differs by spacing only.
  No count change — item 155 was already counted in batch 3.
  Rank 2: 201/348.

  Item 156 (crates/horizon-client-core/src/scoreboard.rs):
  carry-over. The 1.21.5 scoreboard persistence rework:
  `Objective$Packed` / `PlayerTeam$Packed` /
  `Scoreboard$PackedScore` records and `pack()` /
  `loadObjective` / `loadPlayerScore` / `loadPlayerTeam` /
  `packPlayerScores` replace 1.21.4's NBT `savePlayerScores`
  / `loadPlayerScores`. The `Packed` classes are entirely
  1.21.5-only; `ObjectiveCriteria` has no diffs. The Rust's
  `scoreboard.rs::pack()` writes the same top-level keys
  (`Objectives` / `PlayerScores` / `DisplaySlots` / `Teams`)
  and per-entry fields as 1.21.4's `ScoreboardSaveData`
  (`Name` / `DisplayName` / `CriteriaName` / `RenderType` /
  `display_auto_update` / `format`; `Name` / `Objective` /
  `Score` / `Locked` / `display` / `format`; `Name` /
  `DisplayName` / `TeamColor` / `AllowFriendlyFire` /
  `SeeFriendlyInvisibles` / `MemberNamePrefix` /
  `MemberNameSuffix` / `NameTagVisibility` /
  `DeathMessageVisibility` / `CollisionRule` / `Players`) —
  1.21.4 also writes `display_auto_update` and `format`. The
  only difference is the writer's omission rule: the Rust
  omits fields equal to the 1.21.5 codec defaults (empty lists,
  `CriteriaName` dummy, `RenderType` integer,
  `AllowFriendlyFire` / `SeeFriendlyInvisibles` true) where
  1.21.4 writes them always. That is load-equivalent: 1.21.4's
  load guards the booleans with `contains(key, 99)` (default
  true), defaults a missing `CriteriaName` to dummy and a
  missing `RenderType` to integer, and the Rust's `unpack`
  uses the same defaults (dummy, `by_id("")` -> integer,
  booleans true, visibilities Always). Game state round-trips
  identically; only the file bytes differ. Recorded, not
  fixed. `createFormattedDisplayName` / `PlayerTeam` ctor
  differ only by the `HoverEvent$ShowText` record vs
  `HoverEvent(Action, Object)` reshuffle;
  `numberFormatOrDefault` and `Team$Visibility.getDisplayName`
  by spacing only; `Visibility` `byName` / `getAllNames`
  (1.21.4) vs `getSerializedName` (1.21.5) is API — the Rust's
  `by_name` / `name()` matches 1.21.4. Rank 2: 201/348.

  Item 157 (crates/horizon-client/src/game/consume.rs):
  carry-over. `FoodProperties.onConsume` and
  `TeleportRandomlyConsumeEffect.apply` differ only by the
  `Level.playSound` overload (`Entity` in 1.21.5 vs `Player`
  in 1.21.4 as the first argument) — the same sounds at the
  same positions. The Rust's `TeleportConsume::apply` matches
  the shared logic (16 attempts, `(nextDouble - 0.5) *
  diameter` per axis, diameter 16.0 for chorus fruit; a test
  asserts 16 tries). Rank 2: 202/348.

  Item 158 (crates/horizon-client/src/game/options.rs):
  carry-over. `FramerateLimitTracker.getFramerateLimit` in
  1.21.5 is a switch on the factored-out
  `getThrottleReason()`; 1.21.4 inlines the same decision
  tree: iconified -> 10; `inactivityFpsLimit` AFK and idle >
  600000 -> 10, > 60000 -> min(limit, 30); no level and
  (screen or overlay) -> 60; else the option limit. The Rust's
  `window.rs::throttle_reason` / `framerate_limit` matches
  that tree and those constants (`WindowIconified` /
  `LongAfk` -> 10, `ShortAfk` -> min 30, `OutOfLevelMenu` ->
  60). The Rust's extra (`!focused && exclusive_fullscreen`)
  -> `WindowIconified` branch expresses vanilla's
  iconify-on-focus-loss (GLFW iconifies an exclusive-fullscreen
  window when it loses focus, so `isIconified` is true there
  too) — same observable 10 fps cap. `onInputReceived` is
  identical (`latestInputTime` = now). `Options$2.process`
  differs only by `getValue` (1.21.5) vs `getValueOrNull`
  (1.21.4); both return null for a missing key and the
  `process` methods fall back to the default either way.
  `isHeavilyThrottled` / `getThrottleReason` are 1.21.5-only
  names for the factored-out reason; the Rust has its own
  `ThrottleReason` / `heavily_throttled` with the same values.
  Rank 2: 203/348.

  Item 159 (crates/horizon-client/src/mobs/witch.rs):
  carry-over. `Witch.aiStep` differs only by 1.21.5's
  `PotionContents.forEachEffect` taking a duration-scale
  float from the `POTION_DURATION_SCALE` component (default
  1.0); 1.21.4's takes only the consumer. The witch's drink
  path never uses `forEachEffect` — it applies hardcoded
  3600-tick durations for water breathing / fire resistance /
  healing / swiftness — so there is nothing to scale.
  `performRangedAttack` differs by the `SLOWNESS` (1.21.5
  field name) vs `MOVEMENT_SLOWDOWN` (1.21.4 field name)
  rename for the same `minecraft:slowness` effect (the Rust
  uses the id) plus the `playSound` overload. The player-drink
  path's `potion_duration_scale` read is the 1.21.5 component
  with a 1.0 default when absent, matching 1.21.4; the tree
  already notes there is no `potion_duration_scale` key in
  1.21.4. Rank 2: 204/348.

  Item 160 (crates/horizon-client/src/projectiles.rs):
  carry-over. `LlamaSpit.tick`: 1.21.5 discards the spit when
  `isInWater()`, 1.21.4 when `isInWaterOrBubble()` (=
  `isInWater() || isInBubbleColumn()`). The Rust discards the
  spit when `w.in_water(bounds)`, and `in_water` samples the
  fluid table, which maps `minecraft:bubble_column` to
  `Fluid::WATER` (same equivalence as item 139) — so a spit in
  a bubble column is already discarded, matching 1.21.4
  (vanilla's `BubbleColumnBlock.getFluidState` is water too).
  `ThrowableProjectile.handleFirstTickBubbleColumn` differs
  only by 1.21.5's `InsideBlockEffectApplier` parameter to
  `BlockState.entityInside`; the Rust implements the first-tick
  bubble-column push (`first_tick_bubble_column`, tested by
  `a_throwable_spawned_in_a_bubble_column_is_pushed_on_its_first_tick`).
  Rank 2: 205/348.

  Item 161 (crates/horizon-client-core/src/gamerules.rs):
  carry-over. `GameRules`: 1.21.5 adds two rules 1.21.4
  lacks — `allowFireTicksAwayFromPlayer` (UPDATES, default
  false) and `tntExplodes` (MISC, default true), verified by
  counting registered names in the javap dumps (53 vs 55).
  The Rust table holds neither (correct for 1.21.4) and the
  gated behaviours are already 1.21.4-form:
  `can_spread_fire_around` is just `doFireTick` (game.rs,
  level/mod.rs) and `prime_tnt` is ungated (update.rs). The
  table is the 26.x-form table by design (renamed rules) with
  the datafix translating vanilla 1.21.4 saves (fixes_c.rs
  `game_rule_registry` folds `doFireTick` +
  `allowFireTicksAwayFromPlayer` into
  `fire_spread_radius_around_player` and drops
  `spawnChunkRadius`); it covers all 53 vanilla 1.21.4 rules
  (`minecartMaxSpeed` behind the `minecart_improvements`
  flag, default 8) plus four 26.x extras (`pvp`,
  `spawn_monsters`, `spawner_blocks_work`,
  `command_blocks_work`) shared with the parent tree, not a
  1.21.4 delta. `GameRules$Type` / `IntegerValue` /
  `BooleanValue`: 1.21.5's `Type` ctor gains a `Class`
  parameter for `keyCodec` — codec API only. `SpawnData`:
  `WeightedList.codec` vs
  `SimpleWeightedRandomList.wrappedCodecAllowingEmpty` and
  `CompoundTag.read` vs `contains` / `getString` — same data
  shape. `BlockCollisions.computeNext`: jump targets only.
  `CollisionGetter.isUnobstructed`: `VoxelShape.move(Vec3i)`
  vs `move(DDD)` — same. `GameType`: `LEGACY_ID_CODEC`
  added in 1.21.5 — codec only. `LevelReader`:
  whitespace-only plus the 1.21.5-only
  `getHeight(Heightmap.Types, BlockPos)` default, unused in
  the Rust. `WorldDataConfiguration`: `MAP_CODEC` vs `CODEC`
  — codec only. `BlockGetter$BlockStepVisitor` and
  `DryFoliageColor` are new in 1.21.5 (absent from 1.21.4):
  the former unused in the Rust, the latter already handled
  in biome.rs (1.21.4 ships no `dry_foliage.png` and no
  `dry_foliage_color` override; a missing map falls back to
  0x5D4930). Rank 2: 206/348.

  Item 162 (crates/horizon-client/src/game/command_storage.rs):
  carry-over. The Rust already implements the 1.21.4 form:
  per-namespace persistence, `get` returns an empty compound
  when unset, `put` removes on empty contents and marks
  dirty, `keys` lists `ns:path`. The vanilla delta is only
  the persistence backend (1.21.4: per-namespace
  `SavedData.Factory` via `DimensionDataStorage` with
  `DataFixTypes.SAVED_DATA_COMMAND_STORAGE`; 1.21.5:
  `SavedDataType` with a `RecordCodec`). Container save
  writes `{"contents": {...}}` and the file format
  `{"data": {"contents": {...}}, "DataVersion":...}` matches
  the Rust exactly (save.rs:748, save.rs:974,
  block_entity_nbt.rs:492). Known gap, not fixed in this
  batch: vanilla 1.21.4's `createId` is `"command_storage_"`
  + namespace, so vanilla files are
  `data/command_storage_<ns>.dat`, while the Rust writes
  `data/<ns>/command_storage.dat`; the same mismatch exists
  in the 1.21.5 tree, so it is not a 1.21.4 delta.
  Rank 2: 207/348.

  Item 163 (crates/horizon-client/src/game/creaking.rs):
  ported. Two logic fixes in mobs/creaking_heart.rs,
  jar-verified. (1) `creakingHurt`: 1.21.4 spreads resin on
  every hurt (`emitParticles`, then
  `nextIntBetweenInclusive(2,3)` `spreadResin` calls,
  `emitter=100`); 1.21.5 added an AWAKE-state gate before
  the loop. Removed the gate in `creaking_hurt`. (2)
  Protector removal: 1.21.5 keeps a persistent (e.g.
  name-tagged) creaking at daybreak; 1.21.4 removes it
  regardless (removes iff `!isNaturalNight` ||
  `distanceToCreaking() > 34.0` || `playerIsStuckInYou()`).
  Removed the `!persistent` exception and the now-unused
  `persistent` local in `server_tick_update`. Comment fix in
  game/creaking.rs `tick_creaking_heart`: the block-gone path
  is `onRemove` in 1.21.4, not `preRemoveSideEffects` (the
  1.21.5 name). Property form unchanged by design: 1.21.4's
  `CreakingHeartBlock` has `ACTIVE` (boolean) + `NATURAL` +
  `AXIS` (12 states); 1.21.5 replaced `ACTIVE` with the
  `STATE` enum (18 states), verified via javap and the jars'
  `blockstates/creaking_heart.json`. The tree deliberately
  uses the 1.21.5 enum form internally (datafix 4294 renames
  `active` to `creaking_heart_state` on load; the wire
  adapter maps down; pinned 1.21.4 heart states 2917..2928
  are active-major). Known gap, not fixed: the night window
  — 1.21.4 `isNaturalNight` is natural && `skyDarken >= 4`
  (clear-weather window about dayTime [11741, 24259],
  weather-sensitive) while 1.21.5 `isMoonVisible` is
  `dayTime % 24000` in [12600, 23400], weather-blind; the
  Rust's `creaking_active` uses the 1.21.5 range in both
  mobs/creaking_heart.rs and level/creaking.rs. Fixing it
  needs `sky_darken` threaded through `MobWorld` (which
  carries only a `daylight` bool) and the world `Sim` (no
  `sky_darken` available). Also noted:
  level/creaking.rs `heart_tick` uses the 1.21.5
  scheduled-tick form, while 1.21.4 has no block tick
  (`updateShape` schedules none). Rank 2: 208/348.

  Item 164 (crates/horizon-client/src/game/debug_keys.rs):
  carry-over. `TextureManager` diffs: `safeClose` calls
  `texture.releaseId()` after a failed close in 1.21.4
  (1.21.5 only warns); the `DynamicTexture` ctor takes a
  `NativeImage` in 1.21.4 vs a `Supplier` in 1.21.5;
  `dumpAllSheets` checks `RenderSystem.isOnRenderThread()`
  and records a render call when off-thread in 1.21.4, while
  1.21.5 creates the directories directly with an
  `IOException` catch; `registerAndLoad` is slot-only noise.
  The Rust has no `TextureManager` equivalent (wgpu atlases
  own the textures — no GL id to release, no
  `DynamicTexture` class); the only mapped behaviour is
  `DumpDynamicTextures` (F3+S) dumping the GUI atlas via
  hud.rs `dump_contents`, which is version-neutral.
  Rank 2: 209/348.

  Item 165 (crates/horizon-client/src/item_render.rs):
  carry-over. `Direction`: 1.21.5 added a cached `normalVec3f`
  field and `getUnitVec3f()`; 1.21.4's `step()` builds a fresh
  `Vector3f` from `getStepX` / `getStepY` / `getStepZ` each call.
  The values are identical (Down (0,-1,0), Up (0,1,0), North
  (0,0,-1), South (0,0,1), West (-1,0,0), East (1,0,0)), and the
  Rust's `direction_vector` returns exactly those. `getYRot`
  differs only by `ldc` vs `ldc_w` and jump targets (same
  180/0/90/-90 values); `rotate(Matrix4f)` vs
  `rotate(Matrix4fc)` is the interface widening;
  `LEGACY_ID_CODEC` / `LEGACY_ID_CODEC_2D` are 1.21.5
  codec-only; the rest is lambda renumbering.
  Rank 2: 210/348.

  Item 166 (crates/horizon-client/src/mobs/panda.rs):
  carry-over. `PandaRenderer.setupRotations` differs only by
  `mulPose(Quaternionfc)` vs `mulPose(Quaternionf)`; the
  `TEXTURES` map is identical in both versions (normal ->
  panda.png, lazy, worried, playful, brown, weak, aggressive);
  `extractRenderState` is whitespace-only. `Panda.mobInteract`
  is whitespace-only; `addAdditionalSaveData` /
  `readAdditionalSaveData` differ only by codec `store` /
  `read` vs `putString` / `getString` plus `getSerializedName`
  / `byName` (same MainGene / HiddenGene string shape, NORMAL
  default). `Panda$Gene` differs only by the `CODEC` field
  type (`Codec` vs `EnumCodec`). The Rust writes MainGene /
  HiddenGene as serialized-name strings and `gene_from_name`
  maps names back with NORMAL (index 0) as the default — the
  1.21.4 form; `texture()` maps genes to the same seven
  textures and `rotations()` implements `setupRotations`.
  Rank 2: 211/348.

  Item 167 (crates/horizon-protocol/src/item/hash/raw.rs):
  carry-over. `CustomData` differs only by codec shape
  (`TagParser.FLATTENED_CODEC` vs `AS_CODEC`, `getString`
  `Optional` vs `contains`) and whitespace; the Rust hashes
  it as NBT passthrough, version-neutral.
  `TrimMaterial` / `TrimPattern`: the reference path
  (registry id) is correct in both versions and is what
  vanilla servers send. The DIRECT holders differ: 1.21.4's
  `TrimMaterial` stream is (assetName string, ingredient
  holder, overrideArmorAssets map, description NBT) and its
  `TrimPattern` stream is (assetId, template-item holder,
  description, decal), while the Rust reads the 26.3-derived
  forms (material: palette_id string + description; pattern:
  asset_id + description + decal) — the pattern read matches
  1.21.5 exactly (3 fields) and the material read matches
  neither version's stream (1.21.5 writes a base string, an
  overrides map and the description). trim.rs is
  byte-identical between the 1.21.4 and 1.21.5 trees.
  Recorded, not fixed in this batch: the direct path is
  reachable only via inline trim values
  (commands/datapacks), and a correct fix needs
  quirk-branched wire reads in trim.rs, hash/raw.rs and
  skip.rs plus hash-oracle values for direct trims, which no
  local oracle provides. Rank 2: 212/348.

  Item 168 (crates/horizon-client/src/game/mount_menu.rs):
  ported. `HorseInventoryMenu`: 1.21.4's $2 (body) slot
  `isActive` is `canUseSlot(BODY)`, true only for horses and
  llamas (`LivingEntity.canUseSlot` returns false; only
  `Horse` and `Llama` override it, both returning true);
  zombie horses, donkeys, mules, skeleton horses and camels
  get false. The Rust's `can_wear_horse_armor` included
  ZombieHorse (the `CAN_WEAR_HORSE_ARMOR` tag added in 1.21.5
  lists only horse, and 1.21.4 has no such tag), so zombie
  horses wrongly showed a body slot and accepted horse
  armour. Narrowed to Horse only, with the test flipped. The
  saddle slot needs no change: 1.21.4's $1.isActive is
  `isSaddleable()` (alive, adult, tamed; llamas override
  false) and the menu only opens for tamed adult mounts, so
  `shows_saddle` (!llama) is the same rule; the `stillValid`
  armorContainer check is covered by `tick_mount_menu`'s
  alive-and-near check, and the quickMove +1/+2 bound is
  implicit in the Rust's named-slot quick-move.
  `LoomMenu$5.mayPlace`: 1.21.4 is `instanceof
  BannerPatternItem`, 1.21.5 is `has(PROVIDES_BANNER_PATTERNS)`;
  the Rust checked the `minecraft:loom_patterns` tag (absent
  in both versions' data) and the `provides_banner_patterns`
  component (absent in 1.21.4), so the pattern slot accepted
  nothing. `Banners` now also populates `provided` from the
  version's own `pattern_item/*` banner-pattern tags (10
  items in 1.21.4) and `is_pattern_item` is membership in
  `provided`. `LoomMenu$4.mayPlace` is `instanceof DyeItem`
  in both versions; the Rust's `loom_dye` needed the absent
  `loom_dyes` tag and the absent `dye` component, so the dye
  slot also accepted nothing — it now falls back to the
  `<colour>_dye` item id. `StonecutterMenu` differs only by
  the `onCraftedBy` (Level, Player) vs (Player) overload and
  the `playSound` (Player) vs (Entity) overload;
  `LoomMenu$6` likewise. The nautilus does not exist in
  either version (26.x mob), so its mount-menu branches are
  unreachable here. Rank 2: 213/348.

  Item 169 (crates/horizon-client/src/game/glowing.rs):
  carry-over. `EntityRenderer`: 1.21.5 adds four hitbox
  render-state methods (`extractHitboxes` x2,
  `getServerSideEntity`, `extractAdditionalHitboxes`) and
  sets `state.entityType` in `extractRenderState`;
  `renderNameTag` differs only by `mulPose(Quaternionfc)` vs
  `(Quaternionf)`. The Rust's glowing.rs implements only
  `extractRenderState`'s `outlineColor`
  (`isCurrentlyGlowing` / `getTeamColor`), which is identical
  in both versions; the hitbox render-state extraction is a
  1.21.5-only debug feature the Rust does not implement in
  either form, and nothing in glowing reads `entityType`.
  Rank 2: 214/348.

  Item 170 (crates/horizon-client/src/game/level_particles.rs):
  carry-over. `PointedDripstoneBlock`: the static init builds
  the same shapes via `Block.column` in 1.21.5 vs `Block.box`
  in 1.21.4 (`column(6,0,16)` is `box(5,0,5,11,16,11)`)
  with renamed fields (`SHAPE_TIP_MERGE` etc. vs
  `TIP_MERGE_SHAPE`); `getShape` is a switch on
  `DripstoneThickness.ordinal` in 1.21.5 vs an if-chain in
  1.21.4 with the same mapping, and `move(Vec3)` vs
  `move(DDD)`. `getMaxHorizontalOffset` is the
  `MAX_HORIZONTAL_OFFSET` field in 1.21.5 vs an inline
  `0.125f` in 1.21.4 — the field is `SHAPE_BASE.min(X)` =
  0.125f, the same value. `spawnDripParticle`'s Y is `pos.y +
  STALACTITE_DRIP_START_PIXEL` (`SHAPE_TIP_DOWN.min(Y)` =
  5/16) - 0.0625 in 1.21.5 vs `(pos.y + 1) - 0.6875 -
  0.0625` in 1.21.4 — both `pos.y + 0.25`. The Rust writes
  `pos.y + 5.0/16.0 - 0.0625`, matching both versions.
  Rank 2: 215/348.

  Item 171 (crates/horizon-client/src/game/shield.rs):
  ported. `Ravager.blockedByShield` (1.21.4) and
  `blockedByItem` (1.21.5) are bytecode-identical (roarTick 0
  and random < 0.5: stunnedTick 40, `RAVAGER_STUNNED`,
  entity event 39, `blocker.push`; else `strongKnockback` +
  `hurtMarked`); the Rust's `mobs/ravager.rs::blocked`
  implements it, unchanged. The rest is a real gap: the
  `BlocksAttacks` component and its `ItemDamageFunction` do
  not exist in 1.21.4 at all (no class, no `BLOCKS_ATTACKS`
  in `DataComponents`, no `Weapon` component), so the Rust's
  component-driven shield path had nothing to read —
  `raise_shield` returned false and `player_blocking`
  returned None, i.e. shields could not be raised or block.
  Fixed by synthesizing the shield's `blocks_attacks` at
  item load (`BlocksAttacks::shield_1214`: delay 0.25s from
  `getItemBlockingWith`'s 5 ticks, full block within 90
  degrees from `isDamageSourceBlocked`, bypassed by
  `#minecraft:bypasses_shield`, item damage (3.0, 1.0, 1.0)
  from `Player.hurtCurrentlyUsedShield`'s 1 + floor(blocked)
  at blocked >= 3, block/disabled sounds
  `item.shield.block` / `item.shield.break` — exactly the
  1.21.5 shield component's values, so the use duration
  (72000), the block sound and the damage all follow). The
  disable rule is the 1.21.4 one, not the 1.21.5 per-weapon
  seconds: `LivingEntity.canDisableShield` is true for an
  axe held (Warden overrides it to true), and
  `Player.disableShield` applies a flat 100 ticks —
  `shield_disable_seconds` (axe via the `minecraft:axes`
  tag, or warden) replaces the weapon-component read in the
  mob path and in lan_pvp/spear_players. The 0.5 knockback
  is the attacker's (`LivingEntity.blockedByShield` knocks
  the attacker away from the defender, identical in both
  versions; `BlocksAttacks.onBlocked` only plays the sound):
  the Rust knocked the defender back when not fully blocked,
  which is vanilla in neither version — removed, and a
  blocked melee hit now knocks the attacker mob back 0.5 via
  `MobEvent::Knockback` (the ravager's override replaces
  this, as in vanilla). Rank 2: 216/348.

  Item 172 (crates/horizon-client/src/lan/advancements.rs):
  carry-over. `TransmuteRecipe`: 1.21.4's result is a
  `Holder<Item>`, 1.21.5's is a `TransmuteResult` (item +
  count + component patch, new in 1.21.5). `matches`:
  1.21.4 requires exactly 2 non-empty stacks, one
  input-matching stack whose item != the result item and one
  material; 1.21.5 uses `result.isResultUnchanged` (apply
  the result, then same item and components at count 1).
  `assemble`: 1.21.4 is `transmuteCopy(result, 1)` of the
  first such input stack; 1.21.5 is `TransmuteResult.apply`
  = `transmuteCopy` + the component patch. `display`:
  1.21.4 is `ItemSlotDisplay(result holder)`, 1.21.5 is
  `result.display()` (an `ItemStackSlotDisplay`). The Rust
  implements the 1.21.5 form (`matches` via `over()` +
  `same_item_same_components`, i.e. the `isResultUnchanged`
  rule; `assemble` via `Template::over`). For every vanilla
  1.21.4 transmute recipe the two forms agree:
  `material_count` is absent so the bounds default to
  (1, 1) (exactly 2 stacks, as vanilla), the input item
  always differs from the result item (shulker/bundle
  dyeing), and no input is damageable. Recorded, not fixed:
  the 1.21.4 match rule is item-identity, not
  outcome-identity (a datapack recipe with input item ==
  result item matches in neither form's edge), and
  `ItemStack.transmuteCopy` resets damage to 0 for
  damageable results while the Rust's `over()` /
  `transmute()` keep the input's damage — both unreachable
  with vanilla 1.21.4 data. Map cloning is a separate
  `CustomRecipe` in 1.21.4 (1 filled map + 1..8 empty maps),
  modelled by the Rust as the same Transmute arm with
  `add_count`, which matches. Rank 2: 217/348.

  Item 173 (crates/horizon-world/src/level/data.rs):
  carry-over. `WallBlock`: 1.21.5 renamed the wall properties'
  Java fields (`NORTH_WALL` to `NORTH`, `SOUTH_WALL` to
  `SOUTH`, `EAST_WALL` to `EAST`, `WEST_WALL` to `WEST`) and
  refactored `makeShapes` from a `Map<BlockState,
  VoxelShape>` to a `Function<BlockState, VoxelShape>`; the
  state property names are `north` / `south` / `east` /
  `west` in both jars' `cobblestone_wall.json` blockstates,
  and `connectsTo`, `sideUpdate`, `topUpdate`,
  `updateSides`, `shouldRaisePost`, `makeWallState` and the
  shape geometry are identical. The Rust's `wall_shape`
  uses the version-neutral property names via
  `direction_prop` and the `wall_cover` test boxes, so
  nothing is to port. Rank 2: 218/348.

  Item 174 (crates/horizon-protocol/src/packets/play.rs):
  carry-over. `FriendlyByteBuf`: the wire format is
  unchanged (the long-array helpers moved to static methods
  in 1.21.5, `writeBlockHitResult` is identical).
  `HashedStack` and `ProtocolInfo$Details` are new in 1.21.5
  and absent from the 1.21.4 jars: the 1.21.4 wire sends
  full stacks. The Rust already implements the 1.21.4 form
  — `ContainerSetContent` and `ContainerSetSlot` read full
  `Slot` stacks, `ContainerClick`'s `ChangedSlots` reads
  full stacks (with comments noting the 1.21.4 form), and
  the hashed-stack machinery in `item/hash.rs` and
  `hash_ops.rs` is not wired into the 1.21.4 play path.
  Rank 2: 219/348.

  Item 175 (crates/horizon-client-render/src/options_ui.rs):
  carry-over. `GameNarrator`: 1.21.5 only inlined the
  private `narrateMessage` helper (check the narrator is
  active, then `say`) into `say`, `sayChat`, `sayNow` and
  `updateNarratorStatus`; the behaviour is identical. The
  Rust's `game/narrator.rs::update_status` implements
  `updateNarratorStatus` faithfully (clear, say
  "options.narrator : <status>", the `NARRATOR_TOGGLE`
  toast with enabled/disabled/notavailable via
  `horizon_narrator::toast`), and
  `horizon_narrator::update_status` clears and narrates.
  Rank 2: 220/348.

  Item 176 (crates/horizon-client/src/game/spill.rs):
  carry-over, with a comment fix. `AbstractFurnaceBlockEntity`:
  `preRemoveSideEffects` is 1.21.5-only
  (`BlockEntity.preRemoveSideEffects` does not exist in
  1.21.4 at all); in 1.21.4 the same effects run from
  `AbstractFurnaceBlock.onRemove` (`Containers.dropContents`
  plus `getRecipesToAwardAndPopExperience` at
  `Vec3.atCenterOf`), which is exactly what the Rust's
  `spill.rs` does (drop the slots, award the experience
  orbs at the block centre). `RecipesUsed` is a plain
  recipe-id to count compound in both versions' saves
  (1.21.5 writes it via `RECIPES_USED_CODEC`, 1.21.4 builds
  the same compound by hand) and the Rust reads and writes
  that form; the timer loads default to 0 in both
  (`getShort` vs `getShortOr(key, 0)`). Comment fixed:
  `spill.rs` now cites `AbstractFurnaceBlock.onRemove`
  instead of the 1.21.5 method. Rank 2: 221/348.

  Item 177 (crates/horizon-client/src/mobs/ai.rs):
  carry-over. `WaterAnimal.handleAirSupply`: 1.21.4, when
  alive and not `isInWaterOrBubble`, air minus 1, at -20
  reset to 0 and `hurt(drown, 2.0)`, else air 300; called
  unconditionally from `baseTick`. 1.21.5 switches to
  `isInWater`, runs only on the server, and `hurtServer`.
  The Rust's `needs_water` branch in `mobs/ai.rs` does
  exactly the 1.21.4 form inside the alive gate, and its
  fluid scan treats bubble columns as water (the existing
  bubble-column test asserts `eye_in_water` in a bubble
  column), so `isInWaterOrBubble` and `in_water` agree
  wherever a water animal can stand. `Endermite`: the
  constructor sets `life` 0 explicitly (the default),
  `readAdditionalSaveData` is `getIntOr("Lifetime", 0)`
  vs `getInt` (0 when absent), `aiStep` is identical,
  `registerGoals` differs only by formatting.
  Rank 2: 219/348.

  Item 178 (crates/horizon-client/src/mobs/spawning.rs):
  PORTED. `ArmorMaterial.createAttributes` is
  bytecode-identical in both versions (ARMOR plus
  ARMOR_TOUGHNESS plus KNOCKBACK_RESISTANCE when positive,
  id `minecraft:<type>`); only its visibility changed
  (private to public) and `humanoidProperties` /
  `animalProperties` moved out of the class, so the Rust's
  hardcoded horse-armor stats (leather 3, copper 4, iron 5,
  gold 7, diamond 11/2.0) match both. `MoonBrightnessCheck`
  is new in 1.21.5: the 1.21.5 `all_black` cat variant reads
  `moon_brightness >= 0.9` from data, while 1.21.4 picks
  from the `full_moon_spawns` / `default_spawns` tags
  (moon brightness >= 0.9, i.e. phase 0). The natural-spawn
  path already implements the 1.21.4 form
  (`tame::cat_variant`, 11 vs 10 entries), but the
  chunk-generation `Finalize` path read only the data
  registries, which are empty in 1.21.4 (no `pig_variant`,
  `cow_variant`, `chicken_variant`, `cat_variant` or
  `frog_variant` data at all), so every chunk-generated cat
  stayed tabby (variant 0) and every frog stayed temperate.
  Both arms in `mobs/finalize.rs` now fall back to the
  1.21.4 tag forms when the registry is empty: cats pick
  uniformly from 10, or 11 with `all_black` on moon phase 0,
  drawn from the level random; frogs read
  `spawns_warm/cold_variant_frogs` (1/2, else 0). Pigs,
  cows and chickens need no fallback: they have no variant
  in 1.21.4 at all (no class, no data, one pig texture),
  and the 1.21.4 farm-animal climate tags do not exist, so
  the natural-spawn path already leaves them temperate.
  Verified by `cargo check`; the finalize-spawn parity
  harness covers cats and frogs across biomes, reasons and
  moon phases against the 1.21.4 oracle.
  Rank 2: 220/348.

  Item 179 (crates/horizon-assets/examples/item_model_fixture.rs):
  carry-over (re-verification of an item already closed by
  the pre-restart session under report id 189; see the
  ledger correction below). `PalettedPermutations`: 1.21.5
  made the class a record and added an optional `separator`
  field (codec `optionalFieldOf("separator", "_")`); 1.21.4
  has no separator and always names outputs with `"_" +
  key`. The Rust's `palette.rs` reads an optional separator
  defaulting to `"_"`, so on 1.21.4 data (which never
  carries the field) it behaves exactly as 1.21.4; the
  `run` bodies differ only by that separator line and
  lambda renumbering. The pre-restart entry for the same
  item (report id 189) had removed the `"oversized":
  resolved.oversized_in_gui` field from the fixture JSON;
  that angle is preserved in the merged
  `rank2-progress.json` entry.

  Item 180 (crates/horizon-client-render/src/vault.rs):
  carry-over (re-verification of an item already closed by
  the pre-restart session under report id 190; see the
  ledger correction below). `BuiltInLootTables`: the
  loot-table key sets are identical in both jars; 1.21.5
  only refactored the static-init lambdas into a
  `makeDyeKeyMap` helper. `VaultRenderer.render` gained an
  unused `Vec3` camera parameter in 1.21.5 (never loaded in
  the body) and `updateForTopItem` lost a boolean parameter;
  the drawn result (translate 0.5/0.4/0.5, YP spin from
  `rotLerp` of previous/current spin,
  `renderMultipleFromCount`) is identical, and the Rust has
  no `VaultRenderer` drawing path. `VaultState.ejectResultItem`
  and `VaultState$3`/`$4` `onEnter`/`onExit` differ only by
  the `playSound` overload (`Player` in 1.21.4, `Entity` in
  1.21.5, both passed null, both play to all); the Rust
  emits the same events (`Eject` with the triangle spread,
  level event 3017, `block.vault.eject_item` at 1.0 and
  `0.8 + 0.4 * progress`). `PlayerDetector` differs only by
  the `Sheep` package move (`animal.Sheep` to
  `animal.sheep.Sheep`). The pre-restart entry for the same
  item (report id 190) recorded the vault module as
  no-change (0 diff lines between the trees); that angle is
  preserved in the merged `rank2-progress.json` entry.

  LEDGER CORRECTION (2026-11-27, batch 50): the rank-2
  count is corrected 225 -> 220. Batches 46-50 keyed
  `rank2-progress.json` by rank-2 ordinal, but the
  pre-restart entries were keyed by report id
  (id = ordinal + 10), so five items were closed twice and
  counted twice: 162 (legacy id 172), 167 (legacy id 177),
  173 (legacy id 183), 179 (legacy id 189) and 180 (legacy
  id 190). The legacy angles are merged into the
  authoritative session-3 entries and the duplicate keys
  removed; the two legacy entries destroyed by overwrites
  (162's in batch 48, 167's in batch 50) were recovered
  from git history (15407218 and f149e539) and merged the
  same way. After the correction the file reconciles
  exactly: 220 keys, 220 distinct ordinals covered, rank 2
  closed 220/348. The next unclosed ordinals are 151-160,
  then 181-183, 185 onward (184 is already closed via
  legacy id 194). Batches must keep keying by rank-2
  ordinal and must check the report-id key (ordinal + 10)
  before closing an item, so a pre-restart entry is merged
  instead of re-closed.

  LEDGER RECONCILIATION (2026-11-27, Butler, after the batch-50 correction): the batch-50 correction (225 -> 220) found only part of the overcount. Re-verified entry-by-entry against diff-1.21.5-1.21.4/report.json by module/class match: the true rank-2 closed count is 216, not 220. Three ordinals were covered twice under different keys and are now merged into single report-id entries (264 under keys 274+174, 42 under keys 52+80, 59 under keys 69+110+113). Root cause: the file mixes two keying schemes — pre-restart and session 2 keyed by report id (id = ordinal + 10), session 3 keyed batches 46-50 by ordinal, so its keys 161-180 are renumbered +10 to report ids 171-190 in this rebuild. Going forward every entry is keyed by REPORT ID; a batch must look up each item by id in report.json and check the id key before closing it. Batch 51 is ordinals 151-154 (report ids 161-164).

  Batch 51 (2026-11-27): ordinals 151-154 (report ids 161-164).

  Item 151 (report id 161,
  crates/horizon-core/examples/paletted_fixture.rs): ported.
  `PalettedContainer.read` / `PalettedContainer$Data.write`:
  1.21.4 uses `readLongArray` / `writeLongArray` (a VarInt
  word count, then the longs); 1.21.5 uses
  `readFixedSizeLongArray` / `writeFixedSizeLongArray` (no
  count). The horizon-protocol chunk path already handles
  both forms (`Paletted::read` is the 1.21.5 fixed form,
  `read_prefixed` the 1.21.4 VarInt-counted form, used by
  the 1.21.4 adapter `v1_21_4.rs:111`, which normalizes to
  the internal unprefixed form); the gap was only the
  parity fixture `paletted_fixture.rs`, whose `write` /
  `read` emitted and consumed raw longs with NO count (the
  1.21.5 form) while the Java oracle
  `VanillaPalettedContainerOracle` calls vanilla 1.21.4
  `PalettedContainer.write` (prefixed). Fix: the fixture
  `write` now emits `varint(words.len())` before the longs
  (even 0 for bits 0, where the storage holds no words),
  and `read` consumes the VarInt count first and takes
  exactly that many longs. `PalettedContainer::to_wire` /
  `from_wire` need no change (bits, palette, words
  triples; the count is network framing only). Harness:
  `versions/1.21.4/tools/test_paletted_container_parity.py`.
  Rank 2: 217/348.

  Item 152 (report id 162,
  crates/horizon-world/src/snbt_text.rs): ported. 1.21.5
  record-ified `StringTag` (field `value`, `value()` /
  `asString()`, final equals/hashCode) - Java-shape only, no
  Rust impact. Real delta: 1.21.5's
  `quoteAndEscape(String, StringBuilder)` escapes control
  characters via `SnbtGrammar.escapeControlCharacters`
  (\b \f \n \r \t); 1.21.4's single-arg `quoteAndEscape`
  passes control chars through raw, and 1.21.4 has NO
  `SnbtGrammar` class at all (javap-confirmed absent;
  1.21.4's `StringTagVisitor` calls the 1-arg form). Fix:
  removed the `escape_control` call from `quote_and_escape`
  in `snbt_text.rs` (control chars now pass through raw)
  and the now-unused `escape_control` fn; updated the test
  at `crates/horizon-client-core/src/snbt.rs:451` - input
  `"a\nb\\"` now expects `"a<real newline>b\\"` instead of
  the escaped `\n`. Other quote tests unchanged (the
  wrapper-picking rule is identical in both versions).
  Rank 2: 218/348.

  Item 153 (report id 163, widely named:
  world/level/block/entity): carry-over. Skull / Crafter /
  BrewingStand / EnderChest diffs are NBT/codec API-shape
  only (`CompoundTag.read` / `storeNullable` vs `get` /
  `parse`, `getIntOr` / `getShortOr` / `getByteOr` vs
  `getInt` / `getShort` / `getByte` with the same 0
  defaults, `putIntArray(int[])` vs `putIntArray(List)`,
  `playSound` Entity vs Player overload) - the Rust's
  `canonical()` handlers in `block_entity_nbt.rs` already
  match 1.21.4's field sets (skull: profile /
  note_block_sound / custom_name; crafter:
  crafting_ticks_remaining, disabled_slots filtered to 0..9
  sorted/deduped, triggered; brewing: BrewTime,
  total_brew_time 400, Fuel, total_fuel 20,
  speed_multiplier). `FuelValues`: 1.21.5 adds
  SHORT_DRY_GRASS, TALL_DRY_GRASS, LEAF_LITTER fuel
  entries (burnTime/2); javap-confirmed none of the three
  exists as a block or item in 1.21.4, and Rust fuel is
  data-driven from the 1.21.4 registry, so no change.
  `SpawnerBlockEntity$1.setNextSpawnData` flags 260 (1.21.5)
  vs 4 (1.21.4): a server-side `sendBlockUpdated` flag with
  no Rust equivalent path - the Rust emits
  `SpawnerEvent::BlockUpdated` and reloads the client copy;
  the tree already carries the distinction (`flags.rs`
  UPDATE_INVISIBLE=4, UPDATE_NONE=260) and applies
  UPDATE_INVISIBLE where 1.21.5 uses 260 (`growth.rs`,
  `tick.rs`). `BlockEntityType.onlyOpCanSetNbt`: 1.21.4's
  OP_ONLY_CUSTOM_DATA is exactly {COMMAND_BLOCK, LECTERN,
  SIGN, HANGING_SIGN, MOB_SPAWNER, TRIAL_SPAWNER} -
  identical to the Rust's OP_ONLY list in
  `block_entity_item.rs`. SculkCatalyst `bloom` is the
  `playSound` overload only. Recorded, not fixed:
  `TestBlockEntity`, `TestInstanceBlockEntity` (+$1, $Data,
  $Status) and `StructureBlockEntity$1` are ABSENT from the
  1.21.4 jar (javap-confirmed; the report marks them
  removed) and the TEST_BLOCK / TEST_INSTANCE_BLOCK types
  are not registered in 1.21.4, but the tree carries
  test-block / test-instance-block support (TYPES entries +
  canonical handlers in `block_entity_nbt.rs`,
  `test_block.rs`, `TestInstanceBlock` in
  `block_entities.rs`) - unreachable from vanilla 1.21.4
  data (the blocks do not exist in 1.21.4) and the tree's
  own game-test harnesses use it, so removal is a separate
  decision. Rank 2: 219/348.

  Item 154 (report id 164,
  crates/horizon-client/src/first_person.rs): carry-over.
  `ItemRenderer`: `getLayerColorSafe` gained a
  negative-index guard in 1.21.5 (returns -1); 1.21.4 only
  bounds-checks the upper end. The Rust already returns
  None (no tint) for index < 0 or >= tint_count
  (`mesher.rs` `tint_source`) and guards `tint_index >= 0`
  in `lod/mesh.rs` - the hardened behaviour is already the
  Rust's, on the block and item paths alike.
  `renderModelLists` (1.21.4-only), `renderQuadList` /
  `renderItem` / `renderStatic` signature changes
  (BakedModel vs List<BakedQuad>, `tintIndex()` vs
  `getTintIndex()`, the dropped `leftHand` boolean) are
  internal Java refactors; the Rust's wgpu renderer does
  not share the call structure. The `leftHand` boolean is
  stored as `ItemStackRenderState.isLeftHand` in 1.21.4 and
  the field is gone entirely in 1.21.5; the Rust mirrors
  left-hand held items at the transform level
  (`first_person.rs:270-275`, `mirror * right * mirror`;
  `modern_flat_display(left)`) - same observable result.
  `MobEffectUtil`: `hasDigSpeed` /
  `getDigSpeedAmplification` read `MobEffects.DIG_SPEED`
  in 1.21.4 vs HASTE in 1.21.5 - field rename only; the
  registry id is "haste" in BOTH jars (javap: 1.21.4
  `MobEffects` registers `ldc "haste"`; `haste.png` ships
  in the 1.21.4 client assets), and the Rust looks up
  "haste" (`effects.rs:443`, `hand.rs:198`,
  `potions.rs:53`) - correct for 1.21.4. Rank 2: 220/348.

  Batch 52 (2026-11-27): ordinals 155-158 (report ids 165-168),
  all carry-over, no code changes.

  Item 155 (report id 165, widely named:
  world/item/component): carry-over. `ItemAttributeModifiers`:
  1.21.4 has a `showInTooltip` field (`withTooltip`,
  equals/hashCode/toString carry it, `Mutable` passes it
  through `toImmutable`); 1.21.5 dropped it in favour of
  `tooltip_display.hidden_components`. The Rust already
  folds it: kind 13 (`attribute_modifiers`) is in
  `v1_21_4::TOOLTIP_FLAGS`, so the wire decode
  (`v1_21_4.rs:52`) reads the trailing boolean and pushes
  the kind into the synthesized hidden list when false, and
  the save loader (`v1_20_3.rs:270`) folds
  `show_in_tooltip:false` the same way;
  `display.shows("minecraft:attribute_modifiers")` gates
  the lines (`stations.rs:2404`). `compute` /
  `withModifierAdded` / `forEach` are identical modulo the
  constructor call. `BundleContents$Mutable`: `removeOne`
  and `toggleSelectedItem` are semantically identical -
  1.21.5 only extracted the bounds check into
  `indexIsOutsideAllowedBounds` (i < 0 || i >= size); the
  Rust matches (`bundle.rs` `remove_one` maps selected -1
  or >= len to 0, `toggle_selected` deselects when already
  selected or outside). `TooltipProvider`: 1.21.5's
  `addToTooltip` takes an extra `DataComponentGetter` -
  API shape only; the Rust renders tooltips from the full
  stack data. `BlocksAttacks` (+`$DamageReduction`) and
  `InstrumentComponent` are ABSENT from the 1.21.4 jar
  (javap: class not found; the report marks them removed)
  - both 1.21.5-only. `BlocksAttacks` is handled: 1.21.4
  has no `blocks_attacks` component and the tree carries
  blocking on `ItemInfo` instead (`piglins.rs` documents
  it). `InstrumentComponent`'s wire form is a bare holder
  in both versions (1.21.4 `DataComponents.INSTRUMENT` is
  `Holder<Instrument>`; 1.21.5's record has the single
  `EitherHolder` field, its STREAM_CODEC writes just the
  holder) and the Rust decodes a holder (`item.rs:528`).
  Rank 2: 221/348.

  Item 156 (report id 166, widely named:
  world/item/enchantment): carry-over. `ItemEnchantments`:
  1.21.4 has a `showInTooltip` field (`withTooltip`,
  equals/hashCode/toString carry it, `Mutable` carries it
  through `toImmutable`); 1.21.5 dropped it for
  `tooltip_display.hidden_components`. The Rust already
  folds it: kind 10 (`enchantments`) is in
  `v1_21_4::TOOLTIP_FLAGS`, so the wire decode reads the
  trailing boolean into the synthesized hidden list and the
  save loader folds `show_in_tooltip:false` the same way
  (same mechanism as item 155). `EnchantmentInstance`:
  1.21.5 made it a record (equals/hashCode/toString plus
  `level()` / `weight()` / `enchantment()` accessors);
  1.21.4 extends `WeightedEntry$IntrusiveBase` with the
  weight from `Enchantment.getWeight()` - Java-shape only;
  the Rust carries enchantments as (id, level) pairs with
  weights from the registry. `ItemEnchantments$Mutable`
  differs only by the `showInTooltip` plumbing.
  Rank 2: 222/348.

  Item 157 (report id 167, widely named: world/item):
  carry-over. `AdventureModePredicate`: 1.21.4 has a
  `showInTooltip` field (`withTooltip`, equals/hashCode/
  toString carry it); kinds 11 (`can_place_on`) and 12
  (`can_break`) are in `v1_21_4::TOOLTIP_FLAGS`, so the
  fold into the synthesized hidden list already handles it
  (same mechanism as items 155/156). `test()` and
  `areSameBlocks` are identical; `computeTooltip` differs
  only by lambda renumbering. `ItemStack$1`: the stack
  stream codec - 1.21.4 uses `ITEM_STREAM_CODEC`
  (`holderRegistry`) plus the static
  `DataComponentPatch.STREAM_CODEC`, 1.21.5 uses
  `Item.STREAM_CODEC` plus an instance `patchCodec` -
  Java-shape only; the wire form (count varint, 0 = empty,
  item holder, patch) is the same. `DyeColor`: the 16
  values are identical; 1.21.5 only added a
  `LEGACY_ID_CODEC` (byte codec) field, whose job the
  Rust's datafix already does (`fixes_a::dye_color` maps
  legacy numeric ids to names). `Items`: the registry
  differs by exactly 11 items added in 1.21.5 - BLUE_EGG,
  BROWN_EGG, BUSH, CACTUS_FLOWER, DRY_SHORT_GRASS,
  DRY_TALL_GRASS, FIREFLY_BUSH, LEAF_LITTER, TEST_BLOCK,
  TEST_INSTANCE_BLOCK, WILDFLOWERS - and none removed; the
  Rust's item registry is data-driven from 1.21.4's own
  prepared resources so it correctly lacks them, and the
  tree already documents bush / wildflowers / leaf_litter
  as 1.21.5-only with no 1.21.4 tint entry (`mesher.rs:859`).
  `BoatItem`: `getBoat` differs only by the
  `createDefaultStackConfig` overload (`LivingEntity` in
  1.21.5 vs `Player` in 1.21.4, same player passed).
  Recorded, not fixed (same class as item 153's test
  blocks): `animate_tick.rs` has a `firefly_bush` arm
  (`Animate::FireflyBush`, `firefly_bush_sounds`, the idle
  sound) unreachable from vanilla 1.21.4 data, since the
  block does not exist in 1.21.4; the tree already
  documents the 1.21.5-only desert / dry-grass ambient
  sounds as absent. Rank 2: 223/348.

  Item 158 (report id 168,
  crates/horizon-client/src/game/use_hand.rs):
  carry-over. `CakeBlock`: `useItemOn` / `eat` /
  `getAnalogOutputSignal` / `isPathfindable` are identical
  modulo formatting and the `playSound` overload (`Entity`
  in 1.21.5 vs `Player` in 1.21.4, both play to all); the
  shape field was renamed SHAPE_BY_BITE (1.21.4) to SHAPES
  (1.21.5). The Rust implements `CakeBlock.eat` (food 2,
  saturation 0.1) in `level/interact.rs`.
  `CeilingHangingSignBlock`: `getShape` is the same map
  lookup with a default - 1.21.5
  `SHAPES.getOrDefault(rotation, SHAPE_DEFAULT)`, 1.21.4
  `AABBS.get(rotation)` with null falling back to SHAPE;
  the codec lambda and static-init shape construction
  differ only by helper (`Block.box` vs `Block.column`).
  `PlaySoundConsumeEffect.apply` is identical except the
  same `playSound` overload. Rank 2: 224/348.

  Item 70 (report id 80, crates/horizon-client/src/level/fluids.rs):
  carry-over. `BarrierBlock.canPlaceLiquid` / `pickupBlock` and
  `LiquidBlockContainer.canPlaceLiquid`: 1.21.5 changed the first
  parameter from `Player` to `LivingEntity`; bodies behaviorally
  identical - 1.21.4 `if (player == null || !player.isCreative())
  return false/EMPTY; else super...`, 1.21.5 `if (!(user instanceof
  Player p) || !p.isCreative()) return false/EMPTY; else super...`
  (null / non-player / non-creative -> false/EMPTY in both; creative
  player delegates in both). The Rust captures exactly this with a
  `creative: bool` parameter (`fluids.rs` `can_place_liquid` - "only
  a creative player may waterlog"; `pickup_block` - "only a creative
  player drains one"; a dispenser or flowing fluid passes `false`).
  `FlowingFluid.tick`: 1.21.5 uses reference equality (`if_acmpeq`)
  where 1.21.4 uses `Object.equals` - no-op, because `FluidState`
  overrides neither `equals` nor `hashCode` in EITHER jar
  (javap-verified), so `.equals()` IS identity in 1.21.4. The Rust's
  `fluid_tick` (`fluids.rs:641`) compares `FluidState` by value
  (`new_fluid != fluid` - vanilla states are canonical/interned, so
  identity == value equality). `FlowingFluid.canHoldSpecificFluid`
  differs only by the same `canPlaceLiquid` signature.
  Rank 2: 225/348.

  Item 100 (report id 110, crates/horizon-assets/src/item_model.rs):
  carry-over. The entire item is the 1.21.5 `RegistryContextSwapper`
  refactor: `ClientItem` gained `withRegistrySwapper` / 3-arg
  constructor / `registrySwapper()` (1.21.4 is a plain record; codec
  - model + properties - is the same); `ItemModel$BakingContext`
  takes the swapper in 1.21.5 (`contextSwapper()`), 1.21.4 has
  `bake(ResourceLocation)`; `SpecialModelWrapper` takes
  `ModelRenderProperties` in 1.21.5 vs `BakedModel baseModel` in
  1.21.4 (`update` calls `setExtents` +
  `setupSpecialModel(renderer, arg)` + `properties.applyToLayer` vs
  `setupSpecialModel(renderer, arg, baseModel)`);
  `ConditionalItemModel` wraps the property in an
  `ItemModelPropertyTest` (adapted with the swapper) in 1.21.5 vs the
  raw `ConditionalItemModelProperty` in 1.21.4, but `update` logic is
  identical (get property -> pick onTrue/onFalse); `SelectItemModel`
  replaces the `Object2ObjectMap` with a swapper-aware `ModelSelector`
  in 1.21.5 (direct map get in 1.21.4) - same observable selection;
  `SelectItemModel$ModelSelector` is a 1.21.5-only class. The Rust has
  NO `RegistryContextSwapper` anywhere. The Rust decodes the 1.21.4
  JSON schema in `horizon-assets/src/item_model.rs` -
  `minecraft:condition` (property + on_true/on_false) and
  `minecraft:select` (property + cases + fallback, with
  duplicate/empty-case validation) - matching the 1.21.4 codec field
  names. The item_model harness covers the baking.
  Rank 2: 226/348.

  Item 159 (report id 169,
  crates/horizon-client/src/mobs/monster_brain.rs): PORTED. 1.21.5
  added `isAttackTargetStillReachable(Creaking, LivingEntity)` +
  `lambda$initFightActivity$2`, and added
  `NEAREST_VISIBLE_ATTACKABLE_PLAYERS` to `MEMORY_TYPES` (1.21.5's
  list has 11 entries, 1.21.4's has 10 without it). The new check:
  `brain.getMemory(NEAREST_VISIBLE_ATTACKABLE_PLAYERS).map(list ->
  list.contains(target)).orElse(false)`; the fight-activity condition
  is `!isAttackTargetStillReachable(...)` (true when the target is no
  longer visible -> stop attacking). 1.21.5's `initFightActivity`
  takes the Creaking and passes the condition to
  `StopAttackingIfTargetInvalid.create(condition)`, registering via
  `addActivityWithConditions`; 1.21.4's takes only the Brain, uses the
  no-arg `StopAttackingIfTargetInvalid.create()`, and registers via
  `addActivityAndRemoveMemoryWhenStopped(FIGHT, ...,
  ATTACK_TARGET)`. The no-arg create's default condition
  `lambda$create$2` always returns false (javap-verified), so in
  1.21.4 the creaking stops only on the built-ins (`lambda$create$4`:
  `!canAttack`, tired > 200 ticks cant-reach, `!isAlive`, different
  level). The Rust's `B::StopAttacking` non-breeze arm implemented the
  1.21.5 form (stop when the target leaves
  `nearest_visible_targetable_players`) - fixed: the creaking arm now
  uses `false` for the extra condition, with the enum doc corrected.
  The memory slot itself is still populated by the sensor (shared
  sensor memory, also used by other brains) and is no longer read by
  the creaking stop check. Rank 2: 227/348.

  Item 160 (report id 170, crates/horizon-protocol/src/item.rs):
  carry-over. `DataComponentPatch$1`: 1.21.4 is the old anonymous
  `StreamCodec` (encode/decode with `encodeComponent`); 1.21.5
  refactored to `createStreamCodec(CodecGetter)` with $1/$2/$3 as
  `CodecGetter` implementations - Java-shape only. The patch wire form
  is identical in both (readVarInt added count, readVarInt removed
  count, per added: type id + value, per removed: type id - verified
  from 1.21.4 $1.decode), and the Rust implements exactly the 1.21.4
  form (`read_stack` dispatches to `v1_21_4::read_stack` when
  `v1_21_4::reading()`; `v1_21_4::write_stack` writes values with no
  length prefix). `DELIMITED_STREAM_CODEC` ($2) and
  `ItemStack.OPTIONAL_UNTRUSTED_STREAM_CODEC` are 1.21.5-only:
  1.21.4's `ServerboundSetCreativeModeSlotPacket` uses
  `OPTIONAL_STREAM_CODEC` (plain, via `validatedStreamCodec`) -
  javap-verified, 1.21.4's `ItemStack` has no
  `OPTIONAL_UNTRUSTED_STREAM_CODEC`. The Rust's `DelimitedSlot` name
  is misleading but correct for 1.21.4: both its read and write paths
  dispatch to the 1.21.4 plain form when
  `v1_21_4::reading()`/`writing()`. `DataComponentMap`:
  `get`/`getOrDefault`/`getTyped` removed in 1.21.5 (moved to
  `DataComponentGetter`) - Java-shape only; `builder()` differs only
  by javap column alignment. `DataComponentPatch`: `forget`/`split`/
  static lambdas differ only by formatting; 1.21.5-only additions are
  the delimited codec. `DataComponents`: 99 vs 10 methods - 1.21.5
  inlined registrations as per-component lambdas; the component-set
  difference is the 1.21.5-only types (`blocks_attacks` et al., already
  handled in item 155 - the 1.21.4 component id table
  `v1_21_4_ids.rs` has no `blocks_attacks` entry; the Rust's component
  ids are data-driven from 1.21.4's own registry). The 5 removed
  classes (`DataComponentExactPredicate` + `$Builder`,
  `DataComponentPatch$2`, `$3`, `$CodecGetter`) are 1.21.5-only,
  confirmed absent from the 1.21.4 jar (javap: class not found).
  Rank 2: 228/348.

  Item 103 (report id 113, crates/horizon-assets/src/item_model.rs):
  carry-over. `DataComponentPredicate` is a completely different type
  in 1.21.5: 1.21.4 has `public final class DataComponentPredicate
  implements Predicate<DataComponentMap>` (list of TypedDataComponent,
  `test(DataComponentMap)` / `test(DataComponentHolder)`,
  `allOf` / `someOf` / `builder`, `asPatch`, CODEC / STREAM_CODEC /
  EMPTY); 1.21.5 replaced it with `public interface
  core.component.predicates.DataComponentPredicate`
  (`matches(DataComponentGetter)`, `singleCodec`,
  `SINGLE_STREAM_CODEC`, `Map<Type, Predicate>` CODEC / STREAM_CODEC).
  `MinMaxBounds` / `$Doubles` / `$Ints` are identical in both jars (1
  method each, no diff). `EnchantmentsPredicate` (+`$Enchantments`,
  `$StoredEnchantments`) and `MinMaxBounds$1` are 1.21.5-only (javap:
  class not found in the 1.21.4 jar). The Rust has no
  `component_predicate.rs` (the report's file path is stale);
  `item_model.rs` already implements the 1.21.4 form:
  `minecraft:has_component` (with `ignore_default` - patch-only vs
  full components) is decoded and evaluated, and both
  `ConditionProperty` and `SelectProperty` explicitly reject the
  `component` / `ComponentMatches` property as 1.21.5-only ("fails to
  decode here, as vanilla 1.21.4 rejects it"). Rank 2: 229/348.

  Item 168 (report id 178, widely named: world/inventory):
  carry-over. `HorseInventoryMenu` is the 1.21.5 horse-equipment
  rework. 1.21.4 `$1.isActive` is `horse.isSaddleable()` (1.21.5:
  `canUseSlot(SADDLE)` && `type.is(CAN_EQUIP_SADDLE)`); 1.21.4
  `$2.isActive` is `canUseSlot(BODY)` only (1.21.5 adds
  `type.is(CAN_WEAR_HORSE_ARMOR)` || `isLlama`). 1.21.4 `$1.mayPlace`
  is `stack.is(SADDLE) && !hasItem() && isSaddleable()`; `$2.mayPlace`
  is the body-armor fit. 1.21.4 `stillValid` also checks
  `armorContainer.stillValid` (1.21.5 has no `armorContainer` -
  equipment-slot containers instead). 1.21.4 `quickMoveStack` order:
  armor slot (`mayPlace` + empty) -> saddle slot (`mayPlace`) ->
  chest -> other half of the player inventory. The Rust implements the
  1.21.4 form throughout `mount_menu.rs` (`shows_saddle` documented as
  `isSaddleable`, not `canUseSlot` + tag; `can_wear_horse_armor` only
  `Horse`; `body_armor_fits` carpet / horse-armor / nautilus-armor;
  `mount_slot_rules` SADDLE = saddle item, ARMOR = `body_armor_fits`;
  `mount_quick_move` armor-first then saddle then chest then other
  half), and `lan_mount_menu.rs` `lan_mount_valid` covers `stillValid`
  (alive + within range). `StonecutterMenu$2` / `LoomMenu$6` `onTake`
  differ only by the `playSound` overload (`Entity` in 1.21.5 vs
  `Player` in 1.21.4) and `onCraftedBy` gaining a `Level` parameter in
  1.21.5 - API shape only. `LoomMenu$5.mayPlace`: 1.21.5
  `stack.has(PROVIDES_BANNER_PATTERNS)` vs 1.21.4 `instanceof
  BannerPatternItem`; `PROVIDES_BANNER_PATTERNS` is absent from
  1.21.4's `DataComponents` (javap-verified) and `banner.rs` reads the
  `pattern_item/*` tags as the 1.21.4 form.
  `StonecutterMenu.quickMoveStack` differs only by the `onCraftedBy`
  `Level` parameter and branch offsets. Rank 2: 230/348.

  Item 174 (report id 184, widely named: network): carry-over.
  `FriendlyByteBuf`: only the long-array methods differ - 1.21.4
  `readLongArray` / `writeLongArray` are VarInt-counted, 1.21.5 added
  `readFixedSizeLongArray` / `writeFixedSizeLongArray` (fixed-size);
  already handled per item 161 (`Paletted::read` vs `read_prefixed`).
  `writeBlockHitResult` differs only by javap column alignment; all
  other 365+ methods identical. `HashedStack` (+`$1`, `$ActualItem`),
  `ProtocolInfo$Details` (+`$PacketVisitor`),
  `ProtocolInfo$DetailsProvider` are 1.21.5-only (empty 1.21.4 dumps =
  javap class not found). The Rust's `HashedStack` code
  (`item/hash.rs`) is the shared 1.21.5+ implementation; the 1.21.4
  paths send/read whole stacks - `ChangedSlots` reads `Slot` pairs
  with the doc "1.21.4 sends the whole `ItemStack` for each changed
  slot and the carried item (1.21.5 sends `HashedStack` hashes
  instead)", and `remote.rs` sends full stacks for container clicks
  with the same note. Rank 2: 231/348.

  Item 181 (report id 191, crates/horizon-client/src/banner.rs):
  carry-over. `LoomMenu.getSelectablePatterns`: 1.21.5 reads the
  `PROVIDES_BANNER_PATTERNS` component (a `TagKey`) from the pattern
  item; 1.21.4 checks `instanceof BannerPatternItem` and calls
  `getBannerPattern()` (a `TagKey`), then the same `HolderGetter`
  lookup; no item -> the no-item-required patterns in both.
  `LoomMenu.quickMoveStack` has the same component-vs-instanceof
  difference for routing pattern items. The Rust implements the 1.21.4
  form in `banner.rs`: `provided` holds exactly the pattern items
  (from the `pattern_item/*` tags), `is_pattern_item` is membership (=
  the instanceof check, documented against the 1.21.5 component and the
  26.3 `loom_patterns` tag), `selectable` is `getSelectablePatterns`
  (`None` -> `no_item_required`, `Some` -> provided patterns). Loom
  slot order in `menu_clicks.rs` (banner, dye, pattern, result, then
  inventory) matches `LoomMenu`. `BannerItem.getColor` is identical
  modulo formatting. `BannerItem.appendHoverText` /
  `appendHoverTextFromBannerBlockEntityTag` are 1.21.4-only; the Rust
  shows banner patterns via the `minecraft:banner_patterns` component
  path in `item_tooltip.rs` (with show/hide tests). Rank 2: 232/348.

  Item 182 (report id 192, crates/horizon-client/src/game/cartography.rs):
  carry-over. `CartographyTableMenu$5.onTake` differs only by the
  `playSound` overload (`Entity` in 1.21.5 vs `Player` in 1.21.4) and
  `Item.onCraftedBy` gaining a `Level` parameter in 1.21.5 - API shape
  only. `MapRenderer.render` differs only by
  `PoseStack.mulPose(Quaternionfc)` in 1.21.5 vs
  `mulPose(Quaternionf)` in 1.21.4 - same rotation math.
  `CartographyTableMenu.quickMoveStack` differs only by the
  `onCraftedBy` `Level` parameter and branch offsets - same slot logic
  (result slot 2 -> inventory 3..39, slot 1 -> inventory, `MAP_ID`
  check -> slot 0). The Rust's `cartography.rs` implements the menu
  (`map_id` via the `MAP_ID` component, paper zoom, cloning) with the
  same slot layout. Rank 2: 233/348.

  Item 183 (report id 193,
  crates/horizon-client/src/game/storage_alias_tests.rs): carry-over.
  `NbtPathArgument$AllElementsNode.removeTag` / `setTag` differ only
  by `invokeinterface` vs `invokevirtual` on `CollectionTag` methods
  (`size` / `clear` / `addTag` / `stream`) - 1.21.5 made
  `CollectionTag` an interface, 1.21.4 has it as a class; the logic is
  identical (`removeTag` clears the collection and returns its size;
  `setTag` replaces all elements, returning the count). The Rust's NBT
  path implementation (`horizon-client-core/src/nbt_path.rs`, covered
  by `storage_alias_tests.rs`) implements the same all-elements
  semantics. Rank 2: 234/348.

  Item 185 (report id 195, crates/horizon-core/src/geometry/sweep.rs):
  carry-over. `BlockGetter.getBlockStates(AABB)` is identical modulo
  javap column alignment
  (`BlockPos.betweenClosedStream(aabb).map(this::getBlockState)`).
  1.21.4 has `boxTraverseBlocks(Vec3, Vec3, AABB)` returning
  `Iterable<BlockPos>` and `addCollisionsAlongTravel(Set<BlockPos>,
  ...)` filling a set; 1.21.5 replaced them with
  `forEachBlockIntersectedBetween(..., BlockStepVisitor)` and
  `addCollisionsAlongTravel(LongSet, ..., BlockStepVisitor)` returning
  int. The Rust's `sweep_intersected_blocks` implements the 1.21.4
  straight-line traversal (documented in `motion.rs`: "1.21.4
  traverses straight-line (`BlockGetter.boxTraverseBlocks`)" vs
  "1.21.5's `axisStepOrder`"); the bool-returning visitor and step
  index are Rust adaptations (1.21.4's `Iterable` lets callers break
  out of the loop; 1.21.5's `BlockStepVisitor.visit` is void). The
  tiny-movement branch has no deduplication, matching vanilla; the
  moving branch deduplicates via the visited set, matching
  `addCollisionsAlongTravel`'s `Set`. Rank 2: 235/348.

  Item 186 (report id 196, crates/horizon-protocol/src/servers.rs):
  carry-over. `ServerData$ServerPackStatus` has the same 3 values in
  both (ENABLED / DISABLED / PROMPT); 1.21.5 only added `FIELD_CODEC`
  (`MapCodec` from `Codec.BOOL.optionalFieldOf("acceptTextures")`
  `xmap`'d to the enum) - the stored form is the same optional
  boolean. The Rust stores `accept_textures` as `Option<bool>` ("None
  = prompt, Some(true) enabled, Some(false) disabled") and writes /
  reads the `acceptTextures` byte exactly as 1.21.4 does.
  `ServerList.set` / `get` are identical modulo formatting;
  `saveSingleServer` is lambda-renumbered ($0 in 1.21.4 vs $1 in
  1.21.5) and 1.21.5 added a `load$0` lambda for the codec-based load
  path - no difference in the stored `servers.dat` form, which the
  Rust's `load` / `save` / `add_hidden` / `prepend` mirror.
  Rank 2: 236/348.

  Item 187 (report id 197,
  crates/horizon-world/src/worldgen/placement/context.rs):
  carry-over. `ChunkAccess.setBlockState` changed from `(BlockPos,
  BlockState, boolean)` in 1.21.4 to `(BlockPos, BlockState, int)` in
  1.21.5, plus a 2-arg form in 1.21.5 - the flags parameter widened.
  The Rust's placement context documents
  "`ChunkAccess.setBlockState(pos, state, false)` in generation (a
  `ProtoChunk` write with its heightmaps, no neighbour or block-entity
  effects)" and `set_chunk_block` writes the section directly - the
  1.21.4 generation form. `setBlockEntityNbt`: 1.21.5's
  `BlockEntity.getPosFromTag(ChunkPos, CompoundTag)` vs 1.21.4's
  `getPosFromTag(CompoundTag)` (reads x/y/z ints from the tag,
  javap-verified); the rest is identical (`blockEntities` contains ->
  put there, else `pendingBlockEntities`). The Rust reads block-entity
  positions as x/y/z ints from the tag (`block_entity_nbt.rs:375`) -
  the 1.21.4 form. Rank 2: 237/348.

  Item 188 (report id 198, widely named:
  com/mojang/blaze3d/vertex): carry-over. `PoseStack$Pose` is the
  1.21.5 JOML `*fc` (read-only interface) migration: 1.21.5 added
  `computeNormalMatrix` (lazy normal matrix), `setIdentity`, `scale`,
  `translate`, `mulPose(Matrix4fc)`, `rotate(Quaternionfc)`,
  `rotateAround`, and changed `transformNormal` to take `Vector3fc`
  instead of `Vector3f`; 1.21.4's `copy()` uses the copy-constructor.
  Same matrix math in both - the Rust's renderer uses its own matrices
  (glam), not JOML, and the `PoseStack` equivalents in
  `horizon-client-render` (`camera.rs`, `first_person.rs`) implement
  the same translate / rotate / scale / normal behaviour. Same family
  as the `mulPose(Quaternionfc)` diffs in items 182 and 190.
  Rank 2: 238/348.

  Item 189 (report id 199, widely named:
  world/item/equipment/trim): carry-over. `ArmorTrim` is the 1.21.5
  trim rework: 1.21.4 has (material, pattern, `showInTooltip` boolean)
  with `withTooltip` / `showInTooltip` / `hasPatternAndMaterial` and
  `addToTooltip(ctx, consumer, flag)`; 1.21.5 dropped the boolean
  (moved to `tooltip_display.hidden_components`, like the other
  components in items 155-156) and added `layerAssetId(String,
  ResourceKey<EquipmentAsset>)` plus a `DataComponentGetter`
  `addToTooltip`. The Rust handles the 1.21.4 form: TRIM is kind 45,
  which IS in `v1_21_4::TOOLTIP_FLAGS` ([4, 10, 11, 12, 13, 33, 34, 45,
  52]), so the show boolean is folded on read (`v1_21_4.rs:52`) and in
  saves; the TRIM decode reads material + pattern holders (`TrimDirect`
  for direct holders). Trim rendering uses the 1.21.4 layer form
  (`trims.rs`: `textures/trims/entity/<layer>/<pattern>.png` recoloured
  by palette, with equipment-asset `trim_overrides`).
  Rank 2: 239/348.

  Item 190 (report id 200, crates/horizon-client/src/game/game_mode.rs):
  carry-over. `InventoryScreen.init` / `containerTick`: 1.21.4 checks
  `gameMode.hasInfiniteItems()` (javap-verified:
  `localPlayerMode.isCreative()`) where 1.21.5 checks
  `player.hasInfiniteMaterials()` - the Rust gates on the creative
  mode flag (`inventory_after_mode` /
  `mode_changed_under_inventory`: a survival inventory open when the
  player gains infinite materials becomes the creative one, with a unit
  test pinning it), matching 1.21.4. `renderEntityInInventory` differs
  only by `PoseStack.mulPose(Quaternionfc)` vs `(Quaternionf)`.
  `renderEntityInInventoryFollowsMouse` differs only by `ldc` vs
  `ldc_w` (same constants 20.0f and 0.017453292f). Rank 2: 240/348.

  Item 191 (report id 201, crates/horizon-client/src/game/lan_stations.rs):
  carry-over. `EnchantmentMenu`: 1.21.5 record-ified `EnchantmentInstance`
  (`enchantment()` / `level()` accessors vs 1.21.4 fields, same values -
  the record-ification from item 166); playSound overload (`Entity` vs
  `Player`); `clickMenuButton`: 1.21.5 `player.hasInfiniteMaterials()`
  vs 1.21.4 `player.getAbilities().instabuild` (`hasInfiniteMaterials`
  returns `abilities.instabuild`). Rank 2: 241/348.

  Item 193 (report id 203, crates/horizon-client/src/mobs/passengers.rs):
  carry-over. All three diffs are renames / overloads:
  `SkeletonTrapGoal.tick`: `LightningBolt.snapTo(DDD)` in 1.21.5 vs
  `moveTo(DDD)` in 1.21.4 (same positioning call; `snapTo` is the 1.21.5
  rename of `moveTo`). `Husk.doUnderWaterConversion`: `levelEvent`
  overload (`Entity` vs `Player`) only. `Spider.finalizeSpawn`:
  `Skeleton.snapTo(DDDFF)` vs `moveTo(DDDFF)` - same rename. The Rust
  implements the 1.21.4 behaviour: `spring_skeleton_trap` spawns the bolt
  at the horse's position, tames the horse, spawns the skeleton rider +
  3 skeleton horses with riders (`trap_skeleton`: iron helmet unless one
  exists, enchant weapon + helmet always); spider finalizeSpawn has the
  1-in-100 skeleton rider (`next_int_bound(100) == 0`). Rank 2: 242/348.

  Item 194 (report id 204, crates/horizon-client/src/mobs/drops.rs):
  carry-over. `EquipmentSlot`: 1.21.5 added the SADDLE slot (8 values vs
  7 in 1.21.4) plus `canIncreaseExperience()` (false for the saddle).
  The Rust's `EquipmentSlot` has 8 values including Saddle, but every
  1.21.4-observable behaviour matches: `dropPreservedEquipment`
  iterates `EquipmentSlot.VALUES` in both; in 1.21.4 a saddled horse's
  saddle always drops via `AbstractHorse.dropEquipment`'s inventory
  loop (unless `PREVENT_EQUIPMENT_DROP`), and the Rust drops it via the
  preserved path with drop chance 2.0 for "saddle" (`drops.rs:341`) and
  checks `prevents_equipment_drop` at `drops.rs:522` / `554`; XP reward
  "all but the saddle" (`drops.rs:661`, citing `canIncreaseExperience`)
  is the 7 slots in 1.21.4; `EquipmentSlotGroup` "armor" excludes the
  saddle in both. Rank 2: 243/348.

  Item 195 (report id 205, crates/horizon-client/src/mobs/equine.rs):
  carry-over. `AbstractEquineModel`: 1.21.4 has `saddleParts` /
  `ridingParts` fields; `setupAnim` sets `saddleParts[].visible =
  isSaddled` and `ridingParts[].visible = isRidden && isSaddled`.
  1.21.5 removed both fields entirely (0 references) - the saddle
  became an equipment layer (SADDLE slot). The Rust keeps the 1.21.4
  form: `render.rs` pushes the `{name}#saddle` part with the
  per-species saddle texture when `m.extra.saddled && !m.is_baby()`,
  and `anim.rs:1071` shows the reins (`left_saddle_line` /
  `right_saddle_line`) only while ridden. The reins' ridden-only
  condition is harmless: the whole `#saddle` group is not rendered when
  unsaddled. The mesh-definition diffs (`createBodyMesh` /
  `createFullScaleBabyMesh` / constructor) are the same saddle-part
  removal; the Rust bakes the 1.21.4 model JSONs from 1.21.4 assets.
  Rank 2: 244/348.

  Item 196 (report id 206, crates/horizon-client/src/mobs/villager.rs):
  carry-over. `GiveGiftToHero`: 1.21.5's `VillagerProfession` constants are
  `ResourceKey`s (Holder-based) vs 1.21.4's direct `VillagerProfession`
  objects; `getLootTableToThrow` is
  `GIFTS.getOrDefault(profession, UNEMPLOYED_GIFT)` in both, with the same
  13-entry GIFTS map (ARMORER..WEAPONSMITH -> `*_GIFT`). The Rust
  `gift_table` (`villager.rs:391`) implements it: baby -> `baby_gift`,
  none / nitwit -> `unemployed`, else `{profession}_gift`.
  `ResetProfession` `lambda$create$0`: the same condition in both
  (profession not NONE / NITWIT, xp == 0, level <= 1 -> set NONE); the
  1.21.5 form only differs by the `VillagerData` record API
  (`profession()` / `level()` / `withProfession` vs `getProfession()` /
  `getLevel()` / `setProfession`). The Rust `B::ResetProfession`
  (`villager_brain.rs:2126`) implements the 1.21.4 condition exactly.
  Rank 2: 245/348.

  Item 197 (report id 207,
  crates/horizon-client-input/src/keymapping.rs): carry-over.
  `KeyboardHandler.keyPress`: the fullscreen branch in 1.21.4 toggles the
  window, sets the fullscreen option to the window state, calls
  `options.save()`, and updates the `VideoSettingsScreen` button if open;
  1.21.5 drops the save and the button update. The Rust
  `toggle_fullscreen` (`options.rs:655`) flips `settings.fullscreen` and
  calls `settings.save()` - the 1.21.4 save is present; the button update
  is menu chrome (MENU PRINCIPLE) and the option-driven
  `apply_fullscreen` keeps the state. `lambda$handleDebugKeys$0`: 1.21.5
  uses `ClickEvent$OpenFile` vs 1.21.4 `ClickEvent(Action.OPEN_FILE,
  path)` - same file-open result. The rest of `keyPress` is branch
  offsets only. Rank 2: 246/348.

  Item 198 (report id 208, crates/horizon-client-render/src/chat.rs):
  carry-over. `CommandSuggestions$SuggestionsList.mouseScrolled`: 1.21.5
  uses `MouseHandler.getScaledXPos` / `getScaledYPos(Window)`; 1.21.4
  computes `xpos() * guiScaledWidth / screenWidth` manually - the same
  math. The Rust `scroll` (`chat.rs:1968`) implements it: contains
  check, offset clamped to [0, len - line_limit], amount clamped to
  +-1. `FilterMask`: the only diff is the static `FILTERED_STYLE` init -
  `HoverEvent$ShowText` (1.21.5) vs `HoverEvent(Action.SHOW_TEXT, ...)`
  (1.21.4), same style; the Rust `filtered_message` (`chat.rs:23`)
  emits exactly it (dark grey, hover show_text "chat.filtered", `#` per
  masked UTF-16 unit). Rank 2: 247/348.

  Item 199 (report id 209, crates/horizon-client-render/src/effects.rs):
  carry-over. `WebBlock.entityInside`: the bodies are byte-identical in
  both versions (`makeStuckInBlock` with `Vec3(0.25, 0.05, 0.25)`, or
  `(0.5, 0.25, 0.5)` with the WEAVING effect). 1.21.5 only widened the
  signature with the `InsideBlockEffectApplier` parameter (the 1.21.5
  inside-block-effects rework), which the body does not use. The Rust
  `motion.rs:2064` `web_inside` implements the 1.21.4 constants exactly
  (weaving 0.5 / 0.25 / 0.5, else 0.25 / 0.05 / 0.25), and `mobs/ai.rs:934`
  does the same for mobs. Rank 2: 248/348.

  Item 200 (report id 210, crates/horizon-client/src/game/combat.rs):
  carry-over. `CombatTracker`: `recheckStatus` differs only by the static
  `INTENTIONAL_GAME_DESIGN_STYLE` init - `ClickEvent$OpenUrl` /
  `HoverEvent$ShowText` (1.21.5) vs `ClickEvent(Action.OPEN_URL, ...)` /
  `HoverEvent(Action.SHOW_TEXT, ...)` (1.21.4), same URL
  (bugs.mojang.com/browse/MCPE-28723) and hover text. `recordDamage`
  differs only by `LivingEntity.fallDistance` being `double` in 1.21.5
  (`d2f`) vs `float` in 1.21.4 - the same float reaches `CombatEntry`.
  The Rust `combat.rs` uses `fall_distance: f32`, matching 1.21.4.
  Rank 2: 249/348.

  Item 201 (report id 211,
  crates/horizon-client/src/game/commands/data.rs): carry-over.
  `ComparatorBlockEntity.loadAdditional` reads `OutputSignal` with
  `getIntOr(0)` in 1.21.5 vs `getInt` in 1.21.4 - the same 0 default for a
  missing tag; the constructor's explicit `output = 0` in 1.21.5 is the
  field default. The Rust `data.rs` defaults `OutputSignal` to 0 when
  missing (`.unwrap_or(0)`). Rank 2: 250/348.

  Item 203 (report id 213, crates/horizon-client/src/game/machines.rs):
  PORTED. `HopperBlockEntity.loadAdditional`: 1.21.5 reads
  `TransferCooldown` with `getIntOr(-1)`; 1.21.4 reads `getInt`, which is
  0 when the tag lacks the key (javap-verified). The Rust's
  `HopperState::load` used the 1.21.5 -1 default - fixed: the missing-key
  default is now 0, with a comment citing the jar. The -1 stays in
  `HopperState::default` (the constructor sets `cooldownTime = -1`,
  javap-verified) and in `hopper_cooldown`'s no-tag fallback (fresh
  hopper). `isOnCooldown` is `cooldownTime > 0` in 1.21.4, so -1 and 0
  are both "not on cooldown" - the load default only shows in the
  re-saved NBT. `Hopper.isGridAligned` differs only by
  `Block.column(16, 11, 32)` vs `Block.box(0, 11, 0, 16, 32, 16)` - the
  same box. `cargo check -p horizon-client` passed. Rank 2: 251/348.

  Item 204 (report id 214, crates/horizon-client/src/game/mob_gear.rs):
  carry-over. `EquipmentTable` `lambda$static$2`: codec shape only -
  `LootTable.KEY_CODEC` (1.21.5) vs
  `ResourceKey.codec(Registries.LOOT_TABLE)` (1.21.4), the same
  loot-table key codec. `EnchantmentsByCostWithDifficulty.enchant`
  differs only by `EnchantmentInstance` record accessors
  (`enchantment()` / `level()`) vs fields (same as item 166). The Rust
  `mob_gear.rs` resolves equipment tables via
  `spawner::equipment_table`. Rank 2: 252/348.

  Item 205 (report id 215, crates/horizon-client/src/game/pad_craft.rs):
  carry-over. `CraftingMenu.quickMoveStack`: the only behavioural hunk is
  the result-slot `onCraftedBy` call - 1.21.4 calls the 3-arg
  `Item.onCraftedBy(stack, level, player)` via
  `ContainerLevelAccess.execute` (the 1.21.4-only
  `lambda$quickMoveStack$2`), 1.21.5 calls the 2-arg
  `Item.onCraftedBy(stack, player)` directly (the `Level` parameter was
  dropped from `onCraftedBy` in 1.21.5, same as the Stonecutter /
  Cartography diffs in items 182 / 192). The slot routing is identical
  in both (result 0 -> `moveItemStackTo(10, 46, reverse)`, 10..46 ->
  1..10 or 37..46 / 10..37). The Rust's crafting quick-move (`menu.rs`
  `quick_move`, `CraftResult` arm) matches the routing and handles the
  filled-map post-processing (`map_crafted`); the Rust has no
  `onCraftedBy` hook in either form - a pre-existing absence, not a
  1.21.4-vs-1.21.5 delta. Rank 2: 253/348.

  Item 207 (report id 217, crates/horizon-client/src/game/sheep_dye.rs):
  carry-over. `DyeItem.interactLivingEntity`: `Sheep` moved package in
  1.21.5 (`animal/sheep/Sheep` vs `animal/Sheep` in 1.21.4) and the
  playSound overload (`Entity` vs `Player`); the logic is identical
  (alive, not sheared, different colour -> `DYE_USE`, `setColor`,
  shrink). The Rust `mobs::sheep_dye::interact` implements it exactly
  (sheep, alive, not sheared, different colour -> set colour, shrink).
  `tryApplyToSign` differs only by the playSound overload.
  Rank 2: 254/348.

  Item 208 (report id 218, crates/horizon-client/src/game/sign_edit.rs):
  carry-over. `HoneycombItem`: both diffs are the `levelEvent` overload
  only (`Entity` in 1.21.5 vs `Player` in 1.21.4) - `lambda$useOn$2`
  (waxing, level event 3003) and `tryApplyToSign`. The Rust
  `use_sign_applicator` (`sign_edit.rs:231`) implements
  `tryApplyToSign`: waxes any sign (empty or not), plays
  `item.honeycomb.wax_on`, fires level event 3003. Rank 2: 255/348.

  Item 209 (report id 219, crates/horizon-client/src/game/villagers.rs):
  carry-over. `TradeWithVillager`: `figureOutWhatIAmWillingToTrade` and
  `tick` differ only by the `VillagerData` record API -
  `profession().value()` / `Holder.is(FARMER)` in 1.21.5 vs
  `getProfession()` / `== FARMER` in 1.21.4. The logic is identical
  (union of both villagers' `requestedItems`; gossip, then farmer +
  `hasExcessFood` + `wantsMoreFood` -> `throwHalfStack`). The Rust
  `villager_meet` (`villagers.rs:66`) implements the tick (gossip at most
  once in 1200 ticks for both, then surplus food and wanted wares tossed
  over) via `villager::meet`, and `requested_items` feeds the pickup
  scan. Rank 2: 256/348.

  Item 210 (report id 220, crates/horizon-client/src/mobs/breeze.rs):
  carry-over. `LongJump.calculateOptimalJumpVector` differs only by
  `MobEffects.JUMP_BOOST` (1.21.5) vs `MobEffects.JUMP` (1.21.4) - a field
  rename; the 1.21.4 `JUMP` field registers the id "jump_boost"
  (javap-verified: `ldc "jump_boost"` before the field init). The Rust
  `optimal_jump_vector` (`breeze.rs:191`) implements it: shuffled
  allowed angles, first that reaches, plus the jump-boost power `0.1 *
  (amp + 1)` on the normalised Y, looking up "minecraft:jump_boost".
  `start` differs only by the playSound overload. Rank 2: 257/348.

  Item 211 (report id 221,
  crates/horizon-world/src/level/timeline.rs): carry-over.
  `FlowerPotBlock`: the static SHAPE differs only by `Block.column(6, 0,
  6)` vs `Block.box(5, 0, 5, 11, 6, 11)` - the same box. `useItemOn`
  differs only by javap column alignment. `randomTick` differs only by
  the playSound overload. The Rust `potted_eyeblossom_tick`
  (`timeline.rs:113`) implements `randomTick` for potted eyeblossoms
  (open / close toggle by day / night, particles, `open_long` /
  `close_long` sounds). Rank 2: 258/348.

  Item 212 (report id 222,
  crates/horizon-world/src/worldgen/feature_region.rs): carry-over.
  `ProtoChunk.setBlockState` widened from `(BlockPos, BlockState,
  boolean)` in 1.21.4 to `(BlockPos, BlockState, int)` in 1.21.5 - the
  same flags widening as item 197 (`ChunkAccess`). The Rust's
  `feature_region` `set_chunk_block` calls `region.set_block_worldgen`
  (the `ProtoChunk` write with its heightmaps, the 1.21.4 generation
  form). Rank 2: 259/348.

  Item 213 (report id 223, launcher/src/server_list.rs): carry-over.
  `ServerData`: `getResourcePackStatus` / constructor identical modulo
  alignment. `write` / `read` differ only by the codec forms - 1.21.5
  uses `storeNullable` + `BASE64_STRING` for the icon and `FIELD_CODEC`
  for the pack status; 1.21.4 writes the icon as a Base64 string (only
  when non-null) and `acceptTextures` as a boolean (ENABLED -> true,
  DISABLED -> false, PROMPT -> absent), and reads name / ip with
  `getString`, the icon via Base64 decode + `validateIcon` (warn
  "Malformed base64 server icon" on failure), `acceptTextures` via
  `contains(99)` + `getBoolean`. The stored form is the same. The
  launcher's `server_list.rs` keeps entries as raw compounds and writes
  every other field back as read (so `acceptTextures` / `hidden` survive
  round-trips); new entries get name / ip / hidden only (no
  `acceptTextures` = PROMPT); icons are base64-decoded (malformed ->
  dropped, where 1.21.4 logs a warn - logging only). The game's pack
  status itself lives in `servers.rs` (item 196). Rank 2: 260/348.

  Item 214 (report id 224, widely named: world): carry-over.
  `BossEvent$BossBarColor` / `Overlay`: constants identical in both
  (PINK / BLUE / RED / GREEN / YELLOW / PURPLE / WHITE;
  PROGRESS / NOTCHED_6 / 10 / 12 / 20); 1.21.5 only added
  `getSerializedName()` (`StringRepresentable`) and dropped the static
  `byName(String)`. The Rust `boss_overlay.rs` uses numeric color (0..6)
  / overlay (0..4) indices in the same order - no name lookup.
  `Container` `hasAnyMatching` / `countItem`: 1.21.5 iterates via the
  new `iterator()` (`ContainerIterator`, 1.21.5-only), 1.21.4 loops by
  index with `getItem(i)` - same results; `hasAnyOf` identical modulo
  invokedynamic numbering. `Containers`: 1.21.4
  `dropContentsOnDestroy(old, new, level, pos)` drops the contents when
  the block changed and updates the output-signal neighbour; 1.21.5
  split it into `updateNeighboursAfterDestroy` (neighbours only), the
  dropping having moved to the block entity's pre-remove. The Rust
  `spill.rs:585` implements
  `BaseContainerBlockEntity.preRemoveSideEffects`
  (`Containers.dropContents`) on removal, with neighbour updates after -
  the 1.21.4 combined behaviour. Rank 2: 261/348.

  Item 215 (report id 225,
  crates/horizon-client/src/container_screen.rs): carry-over.
  `AbstractContainerScreen.mouseClicked`: 1.21.4 checks
  `gameMode.hasInfiniteItems()`, 1.21.5 checks
  `player.hasInfiniteMaterials()` - the same check as item 200
  (`InventoryScreen`). The Rust `inventory_pick_press`
  (`inventory_input.rs:458`) passes `self.creative` as the `cloning`
  flag to `container_screen` `press`, and `press` documents cloning as
  "`keyPickItem` with infinite materials". Rank 2: 262/348.

  Item 216 (report id 226,
  crates/horizon-client/src/game/copper_golems.rs): carry-over.
  `ChestBlockEntity.playSound` differs only by the playSound overload
  (`Entity` in 1.21.5 vs `Player` in 1.21.4); the pitch / volume maths
  (random 0.9..1.0 range) are identical. Rank 2: 263/348.

  Item 218 (report id 228, crates/horizon-client/src/game/piglins.rs):
  carry-over. `PiglinAi.isWearingSafeArmor`: 1.21.5 iterates
  `EquipmentSlotGroup.ARMOR` slots and calls `getItemBySlot(slot)`;
  1.21.4 iterates `LivingEntity.getArmorAndBodyArmorSlots()` directly -
  the same slots (feet / legs / chest / head / body), and the same check
  (any stack in the `PIGLIN_SAFE_ARMOR` tag -> true). The Rust
  `wearing_gold` (`piglins.rs:226`) iterates `self.armor` and checks the
  `minecraft:piglin_safe_armor` tag. Rank 2: 264/348.

  Item 220 (report id 230, crates/horizon-client/src/mobs/control.rs):
  PORTED. `MoveControl.tick` JUMPING branch: 1.21.4 ends the jump only
  when the mob lands (`onGround` -> WAIT). 1.21.5 added a second end: in
  liquid AND `isAffectedByFluids` -> WAIT (javap-verified both forms;
  `isAffectedByFluids` is protected on `Entity` in 1.21.4 and public on
  `LivingEntity` in 1.21.5). The Rust's Jumping arm used the 1.21.5
  form (`on_ground || is_in_liquid`) - fixed: the liquid clause
  removed, with a comment citing the jar. A 1.21.4 mob that jumps into
  water stays JUMPING until it touches ground. `cargo check
  -p horizon-client` passed. Rank 2: 265/348.

  Item 221 (report id 231,
  crates/horizon-client/src/mod_content_tests.rs): carry-over.
  `StructureStart.loadStaticStart` differs only by the NBT accessor
  forms - `getStringOr("id", "")` vs `getString`, `getIntOr(..., 0)` vs
  `getInt` for ChunkX / ChunkZ / references, `getListOrEmpty("Children")`
  vs `getList("Children", 10)` - all the same defaults ("" / 0 / empty
  list); the "Unknown stucture id" typo is in both. The Rust
  `saved_starts_at` (`generator_structures.rs:448`) implements the
  loading (saved starts with pieces, jigsaw via `PoolPiece::from_saved`,
  fallback to regenerated starts by name + box). Rank 2: 266/348.

  Item 222 (report id 232, crates/horizon-core/src/version.rs):
  carry-over. `SharedConstants.getProtocolVersion` returns 770 in
  1.21.5 vs 769 in 1.21.4 - the version constant itself. The 1.21.4
  tree already pins `PROTOCOL_VERSION = 769`
  (`horizon-core/src/version.rs:9`). Rank 2: 267/348.

  Item 224 (report id 234, crates/horizon-items/src/bundle.rs):
  carry-over. `BundleContents.getWeight` differs only by the bees
  default form - 1.21.5 `Bees.EMPTY` + `bees()` record accessor vs
  1.21.4 `List.of()` - the same empty-list default; a beehive with a
  non-empty bees list weighs 1 in both. The Rust `item_weight`
  (`bundle.rs:73`) implements it: nested bundle -> contents + 1/16,
  beehive with a non-empty `minecraft:bees` list -> 1, else 1 /
  max-stack. Rank 2: 268/348.

  Item 227 (report id 237,
  crates/horizon-world/src/worldgen/generator_structures.rs):
  carry-over. `PiecesContainer.load` differs only by the NBT accessor
  forms - `getCompoundOrEmpty` vs `getCompound`, `getStringOr("id", "")`
  vs `getString` - the same defaults. The logic is identical (lowercase
  id, look up the piece type, log "Unknown structure piece id", load
  each piece, log "Exception loading structure piece with id"). The Rust
  `generator_structures.rs:918` / `1102` implements the load (piece id
  after its `RENAMES`, the four older ids). Rank 2: 269/348.

  Item 228 (report id 238, widely named: world/entity/animal):
  carry-over. `AbstractFish.travel`: the water-movement body is
  identical in both (`moveRelative` 0.01, move SELF, scale 0.9, sink
  -0.005 with no target); 1.21.5 only removed the
  `isControlledByLocalInstance()` gate. On the server the gate is always
  true (no controlling player -> `isEffectiveAi` -> true server-side),
  and the Rust ticks mobs only on the server thread (`creatures.rs`
  `tick_creatures` -> `ai.rs` `server_ai_step` -> `travel`), so the
  Rust's `aquatic::travel` (in-water -> custom movement, 0.01 / 0.9 /
  -0.005) matches 1.21.4 server-side exactly. `defineSynchedData`
  identical (`FROM_BUCKET` false); `readAdditionalSaveData`
  `getBooleanOr(false)` vs `getBoolean`, same default.
  `MushroomCow$Variant`: 1.21.5 added a `DEFAULT` variant + `byId(int)`;
  1.21.4 has RED / BROWN by name - the Rust `render.rs` picks
  `mooshroom_brown` when `extra.variant == 1`, else red.
  `Salmon$Variant`: same (SMALL / MEDIUM / LARGE by name; the Rust
  `render.rs` maps 0 -> small, 2 -> large, else medium).
  `Wolf$WolfPackData` is 1.21.4-only (extends `AgeableMobGroupData`, one
  `type` `Holder<WolfVariant>` field - the spawn group data); 1.21.5
  removed it. The Rust `tame.rs:33` `wolf_variant_for` picks the variant
  by biome, the 1.21.4 form. Rank 2: 270/348.

  Item 229 (report id 239, widely named: world/level/chunk):
  carry-over. `UpgradeData`: 1.21.5 rewrote the serialisation with
  codecs (`BLOCK_TICKS_CODEC` / `FLUID_TICKS_CODEC`, tick loading in the
  constructor); 1.21.4 has the manual `loadTicks` / write lambdas. The
  stored keys are the same (`neighbor_block_ticks`,
  `neighbor_fluid_ticks`, `Indices`, `Sides`). The Rust's datafix layer
  handles the tag form directly (`chunk_flatten.rs:612` writes
  `UpgradeData`, `chunk_leaves.rs:436` reads it, `fixes_cliffs.rs:1524`
  migrates block / fluid_ticks to `neighbor_*`).
  `ChunkSource.updateChunkForced` returns void in 1.21.4 vs boolean in
  1.21.5 and `getForceLoadedChunks` is 1.21.5-only - API shape; the
  Rust's forced chunks live in `streaming.forced` (`levels.rs`).
  `ImposterProtoChunk.setBlockState` is the boolean -> int flags
  widening (same as items 197 / 222). Rank 2: 271/348.

  Item 230 (report id 240, widely named:
  world/level/levelgen/structure/templatesystem): carry-over.
  `StructureTemplate$JigsawBlockInfo.pool()` is a `ResourceLocation` in
  1.21.4 vs a `ResourceKey<StructureTemplatePool>` in 1.21.5 (same
  location string); `of()` reads name / pool / target as strings via
  `getString` + `ResourceLocation.parse` in 1.21.4 vs codec reads with
  `EMPTY` / `EMPTY_ID` defaults in 1.21.5. The Rust `jigsaw.rs:78`
  stores `pool` as a plain name string (`Cow<str>`) - the 1.21.4 form.
  `StructureTemplateManager` `loadFromTestStructures` /
  `listTestStructures` differ only by `testStructuresDir` being a
  `String` (+ `Paths.get`) in 1.21.4 vs a `Path` in 1.21.5 - the same
  test-structures folder; the Rust has no test-structure loading.
  Rank 2: 272/348.

  Item 231 (report id 241, crates/horizon-client-core/src/bossbar.rs):
  carry-over. `CustomBossEvent`: `setPlayers` / `onPlayerDisconnect` /
  `removePlayer` differ only by alignment; `lambda$getDisplayName$0`
  only by the `HoverEvent$ShowText` vs `HoverEvent(Action.SHOW_TEXT,
  ...)` form, same result. The save / load rework is the 1.21.5
  `Packed` record + codec vs the 1.21.4 direct NBT save / load, but the
  stored form is the same: `CustomBossEvents.save` writes a compound
  keyed by the event's text id, each value the event compound with keys
  Name / Visible / Value / Max / Color / Overlay / DarkenScreen /
  PlayBossMusic / CreateWorldFog / Players - exactly what the Rust's
  `BossBars::pack` writes and `unpack` reads (same defaults: Max 100,
  Value 0, color white, style progress). Rank 2: 273/348.

  Item 235 (report id 245, widely named: world/effect): carry-over.
  `MobEffect`: 1.21.4 has a single `blendDurationTicks` field
  (`setBlendDuration(int)`); 1.21.5 split it into blend-in / blend-out
  / blend-out-advance (`setBlendDuration(int, int, int)`). The only
  effect with a blend set is nausea in 1.21.5 (150 / 20 / 60); in 1.21.4
  nausea has none. The Rust has no blend-duration consumer at all (no
  `blendDuration` anywhere; `potions.rs:69` registers nausea with colour
  5578058 only), so the split is unobservable in the Rust.
  `lambda$onEffectAdded$2` differs only by the `playSound` overload
  (Entity vs Player). `MobEffects.bootstrap` differs only by the field
  renames (SPEED -> MOVEMENT_SPEED, SLOWNESS -> MOVEMENT_SLOWDOWN,
  HASTE -> DIG_SPEED, MINING_FATIGUE -> DIG_SLOWDOWN, STRENGTH ->
  DAMAGE_BOOST, INSTANT_HEALTH -> HEAL, INSTANT_DAMAGE -> HARM,
  JUMP_BOOST -> JUMP); the registry ids ("speed", "slowness", ...) are
  unchanged. Rank 2: 274/348.

  Item 236 (report id 246, widely named: world/level/block/state):
  carry-over. `BlockBehaviour`: 1.21.4 `onRemove` removes the block
  entity when the old state has one and the new state's block differs
  (javap-verified); 1.21.5 removed `onRemove` and added the new,
  empty-base `affectNeighborsAfterRemoval` hook. The Rust's
  `game.rs:1780` `remove_block_entity` + `level/update.rs:362`
  implements the 1.21.4 removal flow (`removeBlockEntity` before the
  neighbours hear of the removal). `getEntityInsideCollisionShape`:
  both default implementations return `Shapes.block()` (the whole
  block); only the signature widened (Level -> BlockGetter + Entity).
  The Rust's `motion.rs:1835` / `entities.rs:1057` /
  `falling_block.rs:168` / `arrows.rs:1313` cite it as the whole block.
  `entityInside` is the `InsideBlockEffectApplier` widening (same as
  item 209). `StateHolder`: `getNullableValue` public -> private,
  `hasProperty` generic -> wildcard, `equals` / `hashCode` added - API
  shape; the Rust has its own state model. Rank 2: 275/348.

  Item 238 (report id 248, crates/horizon-assets/src/blockstate/choice.rs):
  carry-over. The entire `MultiPartModel` hierarchy (`MultiPartModel`,
  `Selector`, `SharedBakedState`, `Unbaked`, `Unbaked$1`,
  `Unbaked$1Key`) and `WeightedVariants` are 1.21.5-only - the 1.21.5
  multipart model rewrite (new `BlockStateModel` API). In 1.21.4 the
  multipart model is `MultiPart` (`UnbakedBlockStateModel`,
  `InstantiatedSelector` list, `bake` -> `BakedModel`), and weighted
  variants are the `Choice<Variant>` weighted selection. The Rust
  implements the 1.21.4 form: `bind.rs` evaluates multipart selectors
  by property / value conditions ("unknown multipart property"),
  `choice.rs:83` keeps multipart as `Vec<Choice<T>>` with per-selector
  weighted variants and `nextLong` consumption for a multipart root.
  Rank 2: 276/348.

  Item 241 (report id 251, widely named: advancements/critereon):
  carry-over. `ItemPredicate$Builder`: 1.21.4 has
  `hasComponents(DataComponentPredicate)` and `withSubPredicate`;
  1.21.5 replaced them with `withComponents(DataComponentMatchers)`
  (the `DataComponentPredicate` -> `DataComponentMatchers` rework, same
  as item 113). `EntityPredicate$Builder`: 1.21.5 added
  `components(DataComponentMatchers)`. The Rust implements the 1.21.4
  form: `advancements.rs:771` / `2245` reads `ItemPredicate`
  components / predicates keyed by registry, and `item_model.rs`
  implements the 1.21.4 `DataComponentPredicate` form (item 113).
  Rank 2: 277/348.

  Item 242 (report id 252, widely named: core): carry-over.
  `Direction$Axis.choose(boolean, boolean, boolean)`,
  `SectionPos.sectionToChunk(long)` and
  `HolderGetter$Provider.getOrThrow` are 1.21.5-only additions with no
  1.21.4 counterpart and no Rust consumer. `Vec3i`: the only difference
  is the 1.21.5-added `STREAM_CODEC` (three VarInts); the Rust's
  `buf.rs:497` `VarVec3i` already writes / reads three VarInts - the
  same wire form 1.21.4's `FriendlyByteBuf.writeVec3i` uses.
  Rank 2: 278/348.

  Item 243 (report id 253, widely named: net/minecraft): carry-over.
  `Util`: `blockUntilDone` / `toShuffledList` /
  `createIndexIdentityLookup` / `createIndexLookup` / `sanitizeName`
  differ only by alignment (single-instruction spacing); the rest is
  lambda renumbering from added methods. `makeEnumMap`'s return type
  widened (`EnumMap` -> `Map`); `mapValues` / `mapValuesLazy` /
  `growByHalf` are 1.21.5-only. The Rust has no consumer of any of
  these. `ChatFormatting`: the same 22 constants in both jars
  (javap-verified); 1.21.5 only added a `COLOR_CODEC` field and codec
  lambdas. Rank 2: 279/348.

  Item 244 (report id 254, widely named: util): carry-over.
  `CommonLinks`: the only difference is the 1.21.5-added
  `INTENTIONAL_GAME_DESIGN_BUG` URI
  (https://bugs.mojang.com/browse/MCPE-28723, the same URL as item 210);
  `extendRealms` itself is unchanged and the Rust has no `CommonLinks`
  consumer. `GsonHelper.getNonNull` differs only by alignment (same
  `JsonSyntaxException` "Missing <key>"); `encodesLongerThan` +
  `CountedAppendable` are 1.21.5-only with no Rust consumer.
  `ExtraCodecs`: lambda renumbering, the 1.21.5-only `legacyEnum`
  codec, and the `Matrix4f` -> `Matrix4fc` JOML migration (same as item
  198). `Unit.$values` differs only by alignment. Rank 2: 280/348.

  Item 245 (report id 255, widely named: world/level/levelgen):
  carry-over. `Heightmap$Types`: the same 6 constants in both, same
  usages (WORLDGEN / CLIENT / LIVE_WORLD) and same predicates (NOT_AIR,
  MATERIAL_MOTION_BLOCKING; the two predicate lambdas differ only by
  alignment). 1.21.5 only added an int id in the constructor (for the
  new `STREAM_CODEC`) and id-lookup lambdas. `BelowZeroRetrogen`: the
  codec lambdas differ only by alignment (same keys `target_status` /
  `missing_bedrock`); `replaceOldBedrock` / `applyBedrockMask` differ
  only by the `ProtoChunk.setBlockState` 2-arg vs 3-arg (boolean=false)
  form - the same flags widening as items 197 / 222; `read(CompoundTag)`
  is the 1.21.4 form, the 1.21.5 codec reads the same keys. The Rust's
  datafix reads `below_zero_retrogen` (`fixes_c.rs:899`
  `target_status`). `FlatLevelSource.fillFromNoise` differs only by the
  same `setBlockState` form + branch offsets; the Rust's
  `generator.rs:956` / `961` / `1672` implements the superflat layers
  (`FlatLevelSource.buildTerrain`). Rank 2: 281/348.

  Item 246 (report id 256, crates/horizon-client-render/src/models/catalog.rs):
  carry-over. The `BlockStateModel` hierarchy (`BlockStateModel`,
  `SimpleCachedUnbakedRoot`, `Unbaked`, `UnbakedRoot`) is 1.21.5-only -
  the 1.21.5 model rewrite (new `BlockStateModel` API), the same rewrite
  as item 248. In 1.21.4 block models load through
  `UnbakedBlockStateModel` / `BakedModel`, which the Rust's `catalog.rs`
  implements. `BlockStateModelLoader$LoadedModels` has no methods in
  either version. Rank 2: 282/348.

  Item 248 (report id 258, widely named:
  client/renderer/item/properties/select): carry-over.
  `SelectItemModelProperties.bootstrap`: 1.21.5 registers an additional
  "component" select property type (`ComponentContents.castType()`)
  that 1.21.4 lacks. The Rust's `item_model.rs:651` `SelectProperty`
  implements the 1.21.4 set (`minecraft:has_component`, `charge_type`,
  `display_context`, `local_time`, `context_dimension`, ...) and has no
  "component" value-select type. `Type.create`'s rework (`validateCases`
  extracted, `createCasesFieldCodec`) and the 1.21.5-only `valueCodec()`
  are codec shape. Rank 2: 283/348.

  Item 249 (report id 259, widely named: world/entity/projectile):
  carry-over. `FishingHook.getPlayerOwner` / `handleEntityEvent` are
  identical modulo alignment (owner `instanceof Player`, else null;
  byte 31 client-side local-player hook); the constructor differs only
  by the `snapTo` / `moveTo` rename (same as item 203). `lerpTo` is
  1.21.4-only - the Rust's entity interpolation is generic and
  `fishing.rs` implements cast / reel / loot, not a hook-specific lerp.
  `AbstractHurtingProjectile`: the constructor differs only by
  `snapTo` / `moveTo`; `readAdditionalSaveData` reads
  `acceleration_power` with `getDouble` guarded by `contains(..., 6)`
  in 1.21.4 vs `getDoubleOr(..., 0.1)` in 1.21.5 - the same 0.1 default
  (the field starts at `INITAL_ACCELERATION_POWER`, 0.1, in both; the
  Rust's `lan/entity.rs:196` / `198` documents `accelerationPower`
  0.1). `ThrownLingeringPotion` does not exist in 1.21.4 - lingering
  potions are `ThrownPotion` there (1.21.5 split the class); the Rust's
  `game/potions.rs:54` has the single `ThrownPotion` form
  (`splash_water.rs` cites `AbstractThrownPotion`). Rank 2: 284/348.

  Item 250 (report id 260, widely named: world/scores): carry-over.
  `Score`: 1.21.4 has direct NBT `write(HolderLookup$Provider)` /
  `read(CompoundTag, HolderLookup$Provider)`; 1.21.5 replaced them with
  a `RecordCodecBuilder` codec and a 4-arg constructor (int, boolean,
  `Optional<Component>`, `Optional<NumberFormat>`). The stored fields
  are the same in both: value (`Score`), locked (`Locked`), display
  (`display`), number_format (`format`). The Rust's `scoreboard.rs:704`
  `Score` has exactly those four fields (value, locked, display
  `Option<Comp>`, number_format `Option<NumberFormat>`).
  Rank 2: 285/348.

  Item 251 (report id 261,
  crates/horizon-client/examples/probe/entity_sound_packet_fixture.rs):
  carry-over. `StreamCodec$15` / `$16` are vanilla's internal anonymous
  `StreamCodec` factory classes, not packet code: 1.21.5's `$15` is a
  9-part composite (`Function9` constructor) and `$16` is a new
  memoized lazy codec (`Suppliers.memoize` + `UnaryOperator` factory).
  The numbering shifted because 1.21.5 added a factory; neither changes
  any packet's wire format (a composite just sequences its parts). The
  Rust's `entity_sound_packet_fixture.rs` builds sound packets through
  its own hand-written protocol codec and round-trips them; no
  `StreamCodec` factory is involved. Rank 2: 286/348.

  Item 254 (report id 264, widely named:
  client/renderer/blockentity): carry-over.
  `BlockEntityRenderer.render` gained a `Vec3` camera-position
  parameter in 1.21.5 (and `SkullBlockRenderer`'s two render overloads
  with it); the render bodies are identical otherwise - no differing
  body is listed. The Rust renders skulls / block entities through its
  own wgpu renderer (`block_entity_render.rs`, `block_entity_anim.rs`,
  `block_entity_cem.rs`) and does not mirror the Java renderer
  interface. Rank 2: 287/348.

  Item 255 (report id 265, widely named: world/item/crafting):
  carry-over. `SmithingRecipe.baseIngredient()` returns
  `Optional<Ingredient>` in 1.21.4 vs `Ingredient` in 1.21.5; `matches`
  tests the base via `testOptionalIngredient` in 1.21.4 vs a direct
  test in 1.21.5 - identical for every vanilla smithing recipe (all
  have a base; an empty `Optional` matches anything, which no 1.21.5
  recipe can express). `RecipeManager.lambda$static$1` returns the same
  `Optional` in both (1.21.5 wraps the direct base in `Optional.of`).
  `TransmuteRecipe$Serializer`: 1.21.5's result is a `TransmuteResult`
  record (item + count + `DataComponentPatch`, with its own
  `CODEC` / `STREAM_CODEC`); 1.21.4's result is a plain item `Holder`
  (`Item.CODEC` + `holderRegistry` stream codec). The Rust's
  `crafting.rs` `Transmute` has `result: Template` and builds the
  result as the item with count 1 and the source's components
  (`createWithOriginalComponents`, `crafting.rs:1004`) - the 1.21.4
  form. Rank 2: 288/348.

  Item 256 (report id 266, widely named: client/renderer/block):
  carry-over. `ModelBlockRenderer$Cache`: `getShadeBrightness` is
  identical modulo alignment; `getLightColor` computes the same value -
  the difference is caching (1.21.4 caches per-position in the Cache's
  `colorCache` capped at 100 entries with default return MAX_VALUE;
  1.21.5 caches via a `cachedBrightnessGetter`).
  `AmbientOcclusionRenderStorage` and `CommonRenderStorage` are
  1.21.5-only - the 1.21.5 refactor that extracted the AO calculation
  into a storage class. The Rust renders blocks through its own mesher
  (`mesher.rs:1970` / `2125` / `2144` smooth lighting +
  `lighting::ambient_occlusion` per quad) and does not mirror the Java
  renderer internals. Rank 2: 289/348.

  Item 257 (report id 267, widely named: server): carry-over.
  `ServerScoreboard`: `createData` is the save-data codec rework
  (1.21.4 takes `(CompoundTag, HolderLookup$Provider)`, 1.21.5 takes
  `(ScoreboardSaveData$Packed)`) - the same rework as item 260;
  `dataFactory()` is the 1.21.4 form, 1.21.5 uses a codec lambda.
  `onScoreChanged` differs only by alignment (the `Score.value()` call
  spacing). Rank 2: 290/348.

  Item 258 (report id 268, widely named: util/random): carry-over.
  `Weighted` and `WeightedList` (with `Builder` / `Compact` /
  `Selector`) are 1.21.5-only - 1.21.5 introduced the record-based
  `Weighted` (value + weight) and `WeightedList` API. In 1.21.4 the API
  is `WeightedRandomList` / `WeightedRandom`. The Rust implements the
  1.21.4 form: `spawn_selection.rs:752` `weighted_pick` documents
  "`WeightedRandomList.getRandom` (1.21.5 renamed it `WeightedList`)"
  and implements it (total weight, `nextInt(total)`, walk down
  subtracting weights, first entry where selection < 0);
  `particles.rs:3958` and `frog.rs:732` / `1070` cite
  `WeightedRandom.getRandomItem` the same way. Rank 2: 291/348.

  Item 259 (report id 269, widely named: world/entity/animal/horse):
  carry-over. `AbstractChestedHorse`: 1.21.4 saves the chest inventory
  with a +1 slot offset (the loop starts at inventory slot 1, writes
  `Slot = i - 1`; slot 0 is the saddle) and reads `Slot` back into
  inventory slot `Slot + 1`, then calls `syncSaddleToClients()`;
  1.21.5 writes the raw inventory indices (`Slot = i`) with no offset
  and no sync call. The Rust implements the 1.21.4 form: the saddle is
  stored separately as `SaddleItem` (`entity_nbt.rs:1085`) and
  `m.extra.items` is the 15 chest slots only, written by
  `encode_items` with `Slot` = chest index (`item_nbt.rs:95`) and read
  back by `decode_items(tag, 15)` - byte-identical to 1.21.4's save.
  `SkeletonHorse`: `isTrap` / `setTrap` / `aiStep` / add / read differ
  only by alignment; the constructor's explicit `isTrap=false` /
  `trapTime=0` initialisation is the same values;
  `checkSkeletonHorseSpawnRules` is identical
  (`isBrightEnoughToSpawn`); `trapTime` 18000 and the
  `SkeletonTrapTime` key are the same. `TraderLlama`:
  `readAdditionalSaveData` reads `DespawnDelay` with `getInt` guarded
  by `contains(..., 99)` in 1.21.4 vs `getIntOr(..., 47999)` in 1.21.5 -
  the same 47999 default (the Rust's `llama.rs:97`
  `DEFAULT_DESPAWN_DELAY = 47999`). Rank 2: 292/348.

  Item 260 (report id 270, widely named: world/entity/monster):
  carry-over. `AbstractSkeleton.registerGoals` differs only by the Wolf
  class's package move (`animal/Wolf` in 1.21.4 vs `animal/wolf/Wolf`
  in 1.21.5) - the same `AvoidEntityGoal` (6.0f, 1.0, 1.2).
  `finalizeSpawn`'s Halloween pumpkin logic is identical (Oct 31, 25%
  chance, empty head slot -> pumpkin / carved pumpkin, drop chance 0 -
  written via `armorDropChances[HEAD.getIndex()] = 0` in 1.21.4 vs
  `setDropChance(HEAD, 0)` in 1.21.5). `setItemSlot` (1.21.4) vs
  `onEquipItem` (1.21.5) is the equipment-hook rename; 1.21.4's
  `setItemSlot` calls `Monster.setItemSlot` then server-side
  `reassessWeaponGoal()`, which the Rust's `undead.rs:658` / `704`
  implements (bow goal, hard interval on Hard).
  `Ghast.readAdditionalSaveData` reads `ExplosionPower` with `getByte`
  guarded by `contains(..., 99)` in 1.21.4 vs `getByteOr(..., 1)` in
  1.21.5 - the same default 1. Rank 2: 293/348.

  Item 266 (report id 276, widely named: client/particle): carry-over.
  `ParticleEngine.registerProviders`: 1.21.5 added `TINTED_LEAVES` and
  `FIREFLY` registrations that 1.21.4 lacks (javap-verified: 1.21.4's
  `ParticleTypes` has `PALE_OAK_LEAVES` but no `TINTED_LEAVES` or
  `FIREFLY`; 1.21.5 has both). The Rust registers no tinted_leaves /
  firefly provider and has pale_oak_leaves (`animate_tick.rs:138`) -
  the 1.21.4 set. The `reload` / constructor differences are the
  particles atlas id location (`PARTICLES_ATLAS_INFO` in 1.21.5 vs
  `AtlasIds.PARTICLES` in 1.21.4) and a `TextureManager` field kept in
  1.21.4 - internal. Rank 2: 294/348.

  Item 267 (report id 277, widely named:
  client/renderer/entity/layers): carry-over.
  `EquipmentLayerRenderer$TrimSpriteKey`: 1.21.4 has `textureId()`
  (computed via `getColorPaletteSuffix`) while 1.21.5 has `spriteId()`
  - the trim key changed from a texture id to a sprite id;
  `equals` / `hashCode` / `toString` differ only by the record shape.
  The Rust's `trims.rs` implements the 1.21.4 form: the trim's texture
  is `textures/trims/entity/<layer>/<pattern>.png` recoloured by
  `PalettedTextureManager` with the material's palette
  (`textures/palettes/trim/*`), unless the equipment asset's
  `trim_overrides` pick another texture or palette. Rank 2: 295/348.

  Item 268 (report id 278, widely named: stats): carry-over.
  `ServerRecipeBook.fromNbt` reads "recipes" and "toBeDisplayed" as a
  `ListTag` of strings (`getList(..., 8)`) in 1.21.4 vs via
  `RECIPE_LIST_CODEC` (a list of recipe keys) in 1.21.5 - the same
  stored form (a list of recipe ids); `loadRecipes` takes the `ListTag`
  in 1.21.4 vs the decoded `List` in 1.21.5, same entries.
  `addRecipes` / `removeRecipes` differ only by alignment. The Rust's
  `save.rs:4295` / `4454` reads / writes the `recipeBook` compound via
  `recipe_book::Settings`. Rank 2: 296/348.

  Item 269 (report id 279, widely named:
  world/entity/ai/attributes): carry-over.
  `AttributeInstance.save()` / `load()`: the stored form is the same
  keys in both - id (the attribute's `ResourceLocation` string), base
  (double), modifiers (a list of `AttributeModifier` compounds). 1.21.4
  writes / reads them directly (`putString` / `getDouble` / `getList` +
  `AttributeModifier.save` / `load`); 1.21.5 uses codecs
  (`TYPE_CODEC`, `AttributeModifier.CODEC` list) with the same keys and
  the same 0.0 base default. The Rust's
  `entity_nbt/saved_attributes.rs` reads the same id / modifiers keys.
  Rank 2: 297/348.

  Item 270 (report id 280, widely named:
  world/level/levelgen/structure/pools): carry-over.
  `JigsawPlacement$Placer`: 1.21.4's `readPoolKey` is
  `lookup.lookup(Pools.createKey(info.pool()))` - the pool location
  wrapped in a key plus the alias lookup; 1.21.5 inlines the same
  (`info.pool()` is already a `ResourceKey`) - same resolution, same
  "Empty or non-existent pool" warn on a missing / empty pool. The
  Rust's `jigsaw.rs:1017` / `1107` / `1304` has the aliases map and
  `alias(name)` lookup - the 1.21.4 form.
  `StructureTemplatePool.getTemplates()` is a 1.21.5-only accessor with
  no Rust consumer. Rank 2: 298/348.

  Item 271 (report id 281, crates/horizon-client-input/src/window.rs):
  carry-over. `Window`: the constructor differs only by the
  `DisplayData` record accessors vs fields (1.21.5 record-ified
  `DisplayData`) and `Math.max(width, 1)` vs the equivalent `ifle`
  form - same clamping to >= 1. `updateVsync` differs only by
  `assertOnRenderThread` vs `assertOnRenderThreadOrInit`.
  `FramerateLimitTracker$FramerateThrottleReason` is 1.21.5-only as a
  class, but the Rust's `window.rs` `ThrottleReason` has exactly its
  five values (None, WindowIconified, LongAfk, ShortAfk,
  OutOfLevelMenu) with the same limits (LongAfk > 600s, ShortAfk > 60s,
  iconified / long-AFK -> 10, short-AFK -> min(30), out-of-level menu
  -> 60) and the same vsync present-mode choice. Rank 2: 299/348.

  Item 276 (report id 286, widely named: client/player): carry-over.
  `AbstractClientPlayer`: `isCreative()` / `isSpectator()` are
  1.21.4-only; 1.21.5 replaced them with `gameMode()` returning the
  `GameType`. The Rust uses the 1.21.4 boolean form (a `creative` bool
  passed through the player code, `is_spectator` in
  `entities.rs:2082`), not a `GameType` accessor.
  `getDeltaMovementLerped` (lerp of the previous tick's delta movement)
  and `tick` (saving `deltaMovementOnPreviousTick`, then `Player.tick`)
  differ only by alignment. Rank 2: 300/348.

  Item 277 (report id 287, widely named: sounds): PORTED - the sound-variant
  system does not exist in 1.21.4 at all. javap-verified: 1.21.4's
  `Registries` has no `*_SOUND_VARIANT` fields (1.21.5 has only
  `WOLF_SOUND_VARIANT`); the `WolfSoundVariant` class is absent from the
  1.21.4 jar; `Pig` / `Cow` / `Chicken` / `Cat` / `Wolf` have no
  `DATA_SOUND_VARIANT_ID` and no `getSoundVariant` - only the plain
  `getAmbientSound` / `getHurtSound` / `getDeathSound` / `playStepSound`
  and, for Cat/Wolf, the visual-variant `DATA_VARIANT_ID`. 1.21.5's Pig
  likewise has no sound variant - only its Wolf does. The Rust had
  implemented the 1.21.5+/26.x system: `finalize::sound_variants`
  (pig big/classic/mini, cow classic/moody, chicken classic/picky, cat
  classic/royal, wolf angry/big/classic/cute/grumpy/puglin/sad),
  `variant_sound` building `entity.<name>_<variant>.<kind>` ids (absent
  from 1.21.4's `SoundEvents`: no `entity.wolf_big.*`,
  `entity.cow_moody.*`, `entity.baby_pig.*`, ...), a random
  `sound_variant` draw at `finalizeSpawn` and at spawn, and a
  `"sound_variant"` NBT key on save - so most wolves, pigs, cows,
  chickens and cats played non-existent sound ids. Removed:
  `sound_variants()`, `variant_sound()`, `Finalize::sound()` and the
  `*_sound` `SpawnRegistries` fields (plus the now-unused
  `VariantRegistry::random`), the `Extra.sound_variant` field, the
  `spawning.rs` draw, the `entity_nbt` write/read, the `lan/data.rs`
  `*_sound_variant` registry names, and all eight `variant_sound` call
  sites, now the plain 1.21.4 ids (wolf growl/whine/pant, cat
  purr/purreow/stray_ambient/beg_for_food/eat/hiss, pig/cat eat,
  chicken/cow/pig/wolf step, and the `sounds()` ambient/hurt/death
  triple now `m.kind.sounds`). The farm-animals NBT test now asserts
  no `sound_variant` key is written. The `finalize_spawn` probe fixture
  drops the sound-variant field and the `VanillaFinalizeSpawnOracle`
  drops the `DATA_SOUND_VARIANT_ID` reads (they would throw on
  1.21.4). NOTE (recorded, not fixed): the oracle still uses 1.21.5+
  package names (`animal.pig.Pig` etc. vs 1.21.4's flat `animal.Pig`)
  and the harness paths target the main-repo layout, so the
  finalize_spawn harness is not runnable in this worktree as-is - a
  separate harness-repair job. The `horizon-protocol` `sound_variant`
  skip arms (`item.rs`, `raw.rs`, adapter id tables) are cross-version
  foreign-data tolerance, left alone. Rank 2: 301/348.

  Item 278 (report id 288, widely named: world/entity/animal/frog):
  carry-over. `Tadpole`: all diffs are alignment-only plus NBT accessor
  forms (`getIntOr("Age", 0)` vs `getInt`; `getInt` Optional vs
  `contains` + `getInt` in `loadFromBucketTag`) and the constructor's
  explicit `age = 0` init in 1.21.5 (same as the 1.21.4 field default).
  Age logic identical: `ticksToBeFrog`, `ageUp` x20, `setAge` triggers
  the frog transformation at `ticksToBeFrog`, `aiStep` +1/tick
  server-side, `makeBrain` via `TadpoleAi`. The Rust implements it
  (`creatures.rs`: `Tadpole.ageUp` finalizes the frog for the conversion
  and makes it persistent; `Tadpole.feed` ages it by a tenth of the time
  left to 24000 ticks, growing on its next tick once past 24000; age =
  -24000 for babies). 1.21.4's `Tadpole` javap confirms `ticksToBeFrog`
  + the `age` field + `ageUp(int)`. Rank 2: 302/348.

  Item 279 (report id 289, widely named: world/level/border):
  carry-over. `WorldBorder`: all diffs alignment-only except
  `closestBorder` + `DistancePerDirection` (1.21.5-only new API) and
  `DEFAULT_SETTINGS`'s initialisation site (same values in both: 0, 0,
  0.2, 5.0, 5, 15, 5.9999968E7, 0, 0). The Rust cites
  `WorldBorder.clampVec3ToBound` (`consume.rs:122`, `Mth.clamp`,
  1.0E-5F inside the far edge) and `isWithinBounds`
  (`lan_actions.rs:3376`). Rank 2: 303/348.

  Item 282 (report id 292, crates/horizon-client-render/src/skins.rs):
  carry-over. `SkinTextureDownloader` / `ClientAsset`:
  `lambda$registerTextureInManager$2` - 1.21.5's `DynamicTexture` takes
  a `Supplier<ResourceLocation>` + image, 1.21.4's takes just the image;
  same registration. The Rust's `skins.rs` implements skin/cape/elytra
  registration its own way (`SkinManager.TextureCache.registerTexture`
  at `skins.rs:693`, `SkinManager.registerTextures` at `:832`).
  `ClientAsset` has 0 methods in both. Rank 2: 304/348.

  Item 283 (report id 293, crates/horizon-client-render/src/test_block.rs):
  carry-over. `TestBlock` / `TestBlockMode`: both classes are ABSENT
  from both 1.21.4 jars (javap finds nothing; 1.21.5-only). The Rust's
  `test_block.rs` (`TestBlockMode` Start/Log/Fail/Accept +
  `TestBlock` save form), `BlockEntity::TestBlock` wiring and the
  "minecraft:test_block" NBT mapping are unreachable from vanilla
  1.21.4 data: the blocks and block-entity types are not registered
  in 1.21.4 (see the delta section above) and the three test-block
  packets were removed in commit 65307b5. Removal of the dead support
  is already recorded as needing its own focused item (rank 2:
  166/348 and the later entry). Rank 2: 305/348.

  Item 291 (report id 301, widely named: client/gui/screens):
  carry-over. `DeathScreen`: the only javap diff (52 lines, all
  bytecode offsets) is `ClickEvent.getAction()` (1.21.4, class method)
  vs `ClickEvent.action()` (1.21.5, interface method); no behaviour
  change. The Rust parses click events from JSON into its own
  `ClickEvent` enum (`component.rs` `parse_click_event`; `chat.rs`
  handlers), so no Java API is involved. `LoadingOverlay`: identical
  fields and method signatures in both versions; the only diff is the
  clear path (`GlStateManager._clearColor` / `_clear` in 1.21.4 vs
  `GpuDevice` `CommandEncoder.clearColorTexture` in 1.21.5, same
  colour and effect; the 520 diff lines are offset cascades). The
  Rust's `draw_loading_overlay` fills `BRAND_BACKGROUND`
  (`level_loading.rs:415`). Rank 2: 306/348.

  Item 292 (report id 302, widely named: client/renderer/entity/state):
  carry-over. `SheepRenderState.getWoolColor()` is 1.21.5-only, but
  its formula is exactly 1.21.4's inline computation in
  `SheepWoolLayer.render` (25-tick cycle, `floor(ageInTicks)/25 + id`,
  mod 16, `ARGB.lerp` with `(age%25 + frac)/25`) — the Rust's
  `jeb_wool_rgb` implements it with tests, and id 37 already
  jar-verified the 1.21.4 form. Recorded, NOT fixed: the Rust also
  renders the 1.21.5-only `SheepWoolUndercoatLayer` (class +
  `sheep_wool_undercoat.png` + `sheep_wool.png` are absent from the
  1.21.4 jar; 1.21.4 has only `sheep.png` + `sheep_fur.png`) and its
  wool layer uses the 1.21.5 texture; a sheared dyed sheep keeps the
  tinted undercoat in the Rust (pinned by the test at
  `mobs/render.rs:3411-3433`) but shows no tinted layer in 1.21.4.
  Needs its own focused item (id 37's area; id 37's entry does not
  mention the undercoat). Rank 2: 307/348.

  Item 293 (report id 303, widely named: client/renderer/entity):
  carry-over. `DisplayRenderer` / `ThrownItemRenderer`: the only
  javap diffs are `Quaternionf` -> `Quaternionfc` signature widening
  (10 + 4 diff lines); no behaviour change. The Rust has text-display
  rendering (`game/text_displays.rs`) and thrown projectiles
  (`game/remote_projectiles.rs`, `potions.rs` `remote_thrown_item`).
  Rank 2: 308/348.

  Item 294 (report id 304, widely named: client/renderer/texture/atlas):
  carry-over. `SpriteSource`: the only javap diff is 1.21.4's abstract
  `type()` returning `SpriteSourceType` vs 1.21.5's abstract `codec()`
  returning `MapCodec`; 1.21.4's `SpriteSourceType` is already a Record
  holding a `MapCodec`, so 1.21.5 just inlines it and deletes the class
  (the 1.21.5 dump is empty). The Rust's `atlas_sources.rs` parses the
  five source kinds (`directory`, `single`, `filter`, `unstitch`,
  `paletted_permutations`) from the `atlases/*.json` files directly; no
  Java registration shape is involved, and the JSON schema is unchanged.
  Rank 2: 309/348.

  Item 295 (report id 305, widely named: resources): carry-over.
  `RegistryDataLoader`: the javap diff is entirely bytecode offsets /
  constant-pool renumbering (76 KB vs 88 KB dumps, zero semantic lines).
  No behaviour change; the Rust does not name the class.
  Rank 2: 310/348.

  Item 296 (report id 306, widely named: tags): carry-over. 1.21.5 adds
  five tags and removes none: `EntityTypeTags` `CAN_EQUIP_SADDLE`,
  `CAN_WEAR_HORSE_ARMOR`; `ItemTags` `FLOWERS`, `EGGS`,
  `BOOK_CLONING_TARGET`. The Rust already uses the 1.21.4 form for all
  of them: the saddle slot uses 1.21.4's `isSaddleable()`
  (`mount_menu.rs` `shows_saddle`) not `canUseSlot(SADDLE)` +
  `CAN_EQUIP_SADDLE`, and body armour only for `Species::Horse`,
  matching 1.21.4's `canUseSlot(BODY)`; `EGGS` is already documented
  absent in 1.21.4 (`mob_looting.rs:59`); the item `FLOWERS` tag is
  never looked up (the only `minecraft:flowers` lookup is the block tag
  in `growth.rs` `TreeGrower.hasFlowers`); `BOOK_CLONING_TARGET`
  contains only `minecraft:writable_book` (verified in the 1.21.5 jar
  data) while 1.21.4's `BookCloningRecipe` tests `Items.WRITABLE_BOOK`
  directly — exactly what the Rust does (`crafting.rs:1722`).
  Rank 2: 311/348.

  Item 297 (report id 307, widely named: world/entity/ai/sensing):
  carry-over. `PlayerSensor`: 1.21.5 adds
  `NEAREST_VISIBLE_ATTACKABLE_PLAYERS` (plural list) to `requires()`
  (3 -> 4 modules) and `doTick` (the attackable list is stored, then
  the singular is set from its first element). 1.21.4 writes only
  `NEAREST_PLAYERS`, `NEAREST_VISIBLE_PLAYER` and
  `NEAREST_VISIBLE_ATTACKABLE_PLAYER` (singular). The Rust's four
  brains (monster, villager, warden, animal) all set a
  `nearest_visible_targetable_players` list slot — the 1.21.5 form —
  but it is never read by any behaviour: the only behavioural reader
  of the sensor's output is the singular at `monster_brain.rs:642`
  (creaking `StartAttacking` finder); the plural appears only in the
  debug-dump name tables. Gameplay matches 1.21.4; the slot is
  write-only unobserved state, a cleanup candidate, not a behaviour
  defect. Rank 2: 312/348.

  Item 298 (report id 308, widely named: world/entity/vehicle):
  carry-over. `ContainerEntity`: no semantic diff. `MinecartHopper`:
  1.21.5 adds a `private static final boolean DEFAULT_ENABLED` that is
  never referenced anywhere in the class — a dead constant. The NBT
  read changed shape but not effect: 1.21.4 keeps the constructor's
  `enabled = true` when "Enabled" is absent (`contains` guard), 1.21.5
  uses `getBooleanOr("Enabled", true)` — absent -> true in both. The
  Rust defaults true (`minecarts.rs:3129`
  `c.enabled = flag("Enabled", true)`); the write path (`putBoolean`
  "Enabled") is identical in both versions. Rank 2: 313/348.

  Item 299 (report id 309, widely named: world/level/block/entity/vault):
  carry-over. `VaultBlockEntity$Server`: with constant-pool numbers and
  bytecode offsets stripped, the normalized diff is 29 lines, all
  branch-target renumbering (`iflt` 95 -> 92, `ifne` 111 -> 108, etc.)
  plus one `Object.equals` + `ifne` -> `if_acmpeq` on two `BlockState`
  values; BlockStates are canonical singletons in vanilla, so `==` and
  `.equals` agree. No behaviour change. Rank 2: 314/348.

  Item 300 (report id 310, widely named: world/level/block/piston):
  carry-over. Two pure refactors with identical observable behaviour.
  (a) `MovingPistonBlock.onRemove` in 1.21.4 (if the new state is not
  the same block, call `PistonMovingBlockEntity.finalTick`) is gone in
  1.21.5; instead 1.21.5's `PistonMovingBlockEntity` gains
  `preRemoveSideEffects(pos, state)` which calls `finalTick` (absent
  in 1.21.4). `finalTick` still runs when a moving-piston block is
  replaced; the Rust implements it via the block-change path
  (`update.rs`: old block == moving_piston, block changed ->
  `remove_moving_piston`, i.e. the `finalTick` effect, `progressO <
  1.0 || client-side`, `pistons.rs` `moving_piston_final_tick`).
  (b) `PistonBaseBlock.getShape`: 1.21.4 switches on facing over the
  six `*_AABB` fields; 1.21.5 returns `SHAPES.get(facing)` where
  `SHAPES = Shapes.rotateAll(Block.boxZ(16,4,16))` — the same six
  rotations of the 4-thick slab, so the Direction switch map in
  `PistonBaseBlock$1` disappears with no shape change. Piston shapes
  are parity-tested against the 1.21.4 jar
  (`tests/test_piston_shape_parity.py`). Rank 2: 315/348.

  Item 301 (report id 311, widely named: world/level/gameevent/vibrations):
  carry-over. `VibrationInfo`: the `javap -c` dumps are byte-identical
  (17,000 bytes each, empty diff) — identical fields, signatures and
  bytecode. The class files differ in md5 only, i.e. debug /
  constant-pool layout, with no semantic change. Rank 2: 316/348.

  Item 302 (report id 312, widely named: world/level/levelgen/carver):
  carry-over. `WorldCarver`: the only real change is
  `ChunkAccess.setBlockState(pos, state, false)` (3-arg, `isMoving =
  false`) in 1.21.4 -> `setBlockState(pos, state)` (2-arg) in 1.21.5 —
  the same call with the moving flag false in both forms; the rest of
  the normalized diff is branch-target renumbering. The Rust's carver
  is mask-based (`horizon-core` `CarvingMask`) with no moving flag in
  its apply path. Rank 2: 317/348.

  Item 303 (report id 313, widely named: world/level/levelgen/structure):
  carry-over. `StructureCheck.loadStructures` rewritten from the 1.21.4
  NBT API (`contains("structures", 10)` + `getCompound`,
  `getString` / `getInt`) to the 1.21.5 Optional-returning API
  (`getCompound` -> `Optional`, `getStringOr` / `getIntOr`,
  `asCompound` / `ifPresent`) — same logic: absent structures / starts
  compounds -> skip, "INVALID" id skipped, "references" defaults to 0.
  The Rust's `saved_structure_starts` (`chunk.rs:637`) uses the 1.21.4
  form (absent -> `None`). Rank 2: 318/348.

  Item 304 (report id 314, widely named: world/level/material):
  carry-over. `FluidState`: 1.21.5 adds `getAABB` / `entityInside` as
  thin delegates to new `Fluid` methods (base `Fluid.entityInside` is
  a no-op; `WaterFluid` applies `EXTINGUISH`, `LavaFluid` applies
  `LAVA_IGNITE` via `InsideBlockEffectApplier`). 1.21.4 has no
  `entityInside` / `getAABB` on `Fluid` or `FluidState` at all — the
  effects live in entity-tick code (`lavaHurt` ignites 15 s, water
  clears fire), which is how the Rust applies them (`entities.rs`
  documents "there is no lavaIgnite in 1.21.4"). No `FluidState` hook
  exists in the Rust, and none is needed for 1.21.4. Rank 2: 319/348.

  Item 305 (report id 315, widely named: world/level/storage/loot):
  carry-over. `LootTable`: the normalized diff is only the `KEY_CODEC`
  addition (a `Codec` for loot-table resource keys, initialized in the
  static block) plus static-init reordering and lambda renumbering
  (`createStackSplitter` $5 -> $6). Loot generation (pools, rolls,
  context) is untouched; the Rust reads loot tables from JSON data by
  id and needs no key codec. Rank 2: 320/348.

  Item 306 (report id 316, widely named: world/phys/shapes): carry-over.
  `ArrayVoxelShape.getCoords(Axis)`: a pattern-match switch refactor in
  1.21.5 (same axis -> coords mapping; the new form throws `MatchException`
  on the unreachable default). `DiscreteVoxelShape.rotate(OctahedralGroup)`
  is NEW in 1.21.5 (identity returns self, else permutes axes via
  `OctahedralGroup.permute` and rebuilds), and `Shapes.rotate` /
  `rotateAll` / `rotateHorizontalAxis` / `rotateAllAxis` are all new in
  1.21.5 — 1.21.4's `Shapes` has no rotate methods at all, and the Rust's
  `shape.rs` (`horizon-core` geometry) has no rotation either. Correctly
  absent for 1.21.4. Rank 2: 321/348.

  Item 309 (report id 319, crates/horizon-client/src/dialog.rs):
  carry-over. `StringTag$1`: 1.21.4's `TagType` interface has a default
  `isValue()` returning false, and `StringTag$1` overrides it to return
  true. 1.21.5 deletes `isValue()` from the `TagType` interface entirely,
  taking the override with it. Java-side API removal with no NBT
  data-format change; the Rust's NBT implementation has no `isValue`
  concept. Rank 2: 322/348.

  Item 322 (report id 332, widely named: client/main): carry-over.
  `Main`: the only change is the added `--renderDebugLabels` CLI flag
  (all other options identical and in the same order, just shifted by
  one in the dump). The Rust does not implement the 1.21.5-only flag —
  correct for 1.21.4. Rank 2: 323/348.

  Item 323 (report id 333, widely named: client/model/geom): carry-over.
  `ModelPart$Cube`: the only diff is
  `PoseStack$Pose.transformNormal(Vector3f, Vector3f)` ->
  `transformNormal(Vector3fc, Vector3f)` — the first parameter widened
  from `Vector3f` to the `Vector3fc` interface, the same pattern as
  batch 73's `Quaternionf` -> `Quaternionfc` widening. No behaviour
  change; the Rust has its own math and no JOML calls. Rank 2: 324/348.

  Item 324 (report id 334, widely named:
  client/renderer/item/properties/conditional): carry-over.
  `ConditionalItemModelProperties`: the normalized diff is 7 lines, all
  branch-target renumbering — the bootstrap list (`custom_model_data`,
  `using_item`, `broken`, ...) is identical in both versions. No
  behaviour change. Rank 2: 325/348.

  Item 325 (report id 335, widely named: com/mojang/math): carry-over.
  `Quadrant`: the class does not exist in 1.21.4 at all (empty 1.21.4
  dump) — a new 1.21.5 enum (`R0` / `R90` / `R180` / `R270` + `shift`,
  with a `CODEC`) used by the new
  `OctahedralGroup.fromXYAngles(Quadrant, Quadrant)`, part of the same
  1.21.5 shape-rotation API family as item 316
  (`DiscreteVoxelShape.rotate`). The Rust has no `Quadrant`; its
  rotation is the 1.21.4-form `Rotation` enum
  (`worldgen/structure.rs`). Correctly absent for 1.21.4.
  Rank 2: 326/348.

  Item 326 (report id 336, widely named:
  com/mojang/realmsclient/client/worldupload): carry-over.
  `RealmsCreateWorldFlow`: the normalized diff is 12 lines, all
  branch-target renumbering. No behaviour change. Rank 2: 327/348.

  Item 327 (report id 337, widely named: commands/arguments/blocks):
  carry-over. `BlockStateParser`: the only change is
  `new TagParser(reader).readStruct()` in 1.21.4 ->
  `TagParser.parseCompoundAsArgument(reader)` in 1.21.5 — the same parse
  via a static-method refactor of the `TagParser` API. The Rust's
  `horizon-command` `block.rs` parses the NBT tail (`OpenNbt` state,
  `nbt: Option<String>`) with its own parser. Rank 2: 328/348.

  Item 328 (report id 338, widely named: core/particles): ported.
  1.21.5 adds `TINTED_LEAVES` and `FIREFLY` particle types and the new
  `TintedParticleLeavesBlock`; 1.21.4's `LeavesBlock.animateTick` only
  drips water when raining above (1/15, `DRIPPING_WATER`), and its
  `LeavesBlock` is concrete with no `leafParticleChance` field
  (`TintedParticleLeavesBlock` is absent from both 1.21.4 jars;
  cherry/pale-oak/azalea leaves are plain `LeavesBlock`). The Rust
  spawned falling `TintedLeaves` (0.01) from oak/birch/jungle/acacia/
  dark_oak/mangrove/azalea, `CherryLeaves` (0.1) from cherry and
  `PaleOakLeaves` (0.02) from pale oak — all wrong for 1.21.4. Fixed:
  falling `None` for every 1.21.4 leaf block (the rain drip was already
  correct). The `TintedLeaves`/`Firefly`/`CherryLeaves`/`PaleOakLeaves`
  particle types and renderers stay (decode/render-only;
  `CherryLeaves`/`PaleOakLeaves` are valid 1.21.4 particles a server
  can send). The dead red/orange/yellow poplar arms stay (no poplar
  blocks in 1.21.4) — recorded, not removed. Rank 2: 329/348.

  Item 329 (report id 339, widely named: core/registries): carry-over.
  `Registries`: the field sets are identical between versions
  (sorted-set diff empty); the 453-line normalized diff is pure
  field-declaration reorder. No behaviour change. Rank 2: 330/348.

  Item 330 (report id 340, widely named: data/registries): ported.
  1.21.4's `RegistryDataLoader.SYNCHRONIZED_REGISTRIES` is exactly 12
  entries, verified by parsing the static-init bytecode (`BIOME`,
  `CHAT_TYPE`, `TRIM_PATTERN`, `TRIM_MATERIAL`, `WOLF_VARIANT`,
  `PAINTING_VARIANT`, `DIMENSION_TYPE`, `DAMAGE_TYPE`,
  `BANNER_PATTERN`, `ENCHANTMENT`, `JUKEBOX_SONG`, `INSTRUMENT`). The
  Rust's `lan/data.rs` list had 26; trimmed the 11 that do not exist in
  1.21.4 or are not synced by it (pig/cow/chicken/zombie-nautilus
  variants, `sulfur_cube_archetype`, `test_environment`,
  `test_instance`, `timeline`, `decorated_pot_pattern`,
  `block_transformer`, `worldgen/block_state_provider` — the last four
  are read from prepared data, never from the synced `Dynamic`). Kept
  `frog_variant`, `cat_variant` and `world_clock` with a comment: the
  Rust's own client resolves variant names and the clock only from
  synced registries (`Dynamic::name` has no builtin fallback), so the
  LAN server must sync them for its own client; vanilla 1.21.4 syncs
  none of the three. Runtime-harmless either way (the sync loop skips
  entries absent from the kept registries). Rank 2: 331/348.

  Item 331 (report id 341, widely named: network/syncher): ported.
  `EntityDataSerializers`: `apply_registry_data` used entity-data
  indices matching neither 1.21.4 nor 1.21.5 — every variant index was
  +1 (Cat 20 vs 19, Wolf 23 vs 22, Frog 18 vs 17; Wolf anger 22 vs
  21), so cat/wolf/frog variants and wolf anger never applied from
  `SetEntityData`. Verified by counting `defineId` calls per class in
  the 1.21.4 jars (Entity 0-7, LivingEntity 8-14, Mob 15, AgeableMob
  16, `TamableAnimal` adds `DATA_FLAGS_ID` + `DATA_OWNERUUID_ID`;
  `TameableAnimal` does not exist — the class is spelled
  `TamableAnimal`). Fixed the indices, removed the 1.21.5-only
  pig/cow/chicken/zombie-nautilus arms, and narrowed
  `saves_variant_name` to Wolf/Cat/Frog (only those three write a
  `variant` string in 1.21.4's `addAdditionalSaveData`;
  `VariantUtils.writeVariant` has 0 callers in 1.21.4; Pig/Cow/Chicken
  do not write it). New test pins the 1.21.4 indices. Remaining
  residue recorded, not fixed: `species_variant_component` still
  reports pig/cow/chicken/zombie-nautilus variant components
  (`drops.rs` `chicken_variant` is set from it — harmless, no 1.21.4
  loot table conditions on it) and `SpawnRegistries` still loads the
  absent pig/cow/chicken variant registries (empty in 1.21.4,
  `variant_index` falls back to 0). Rank 2: 332/348.

  Item 332 (report id 342, widely named: server/commands): carry-over.
  `SummonCommand`: the only change is `Entity.moveTo(DDDFF)` in 1.21.4
  -> `Entity.snapTo(DDDFF)` in 1.21.5 — a pure rename, the two bodies
  are identical (`setPosRaw` + `setYRot` + `setXRot`). The Rust's
  summon positions via `place_summoned` (`Pos` + `Rotation` in the NBT
  before spawn). No behaviour change. Rank 2: 333/348.

  Item 333 (report id 343, widely named: world/entity/ai/behavior):
  carry-over. `VillagerMakeLove`: the same `moveTo` -> `snapTo`
  rename, identical bodies. The Rust spawns the baby at the parent's
  position with age -24000 (`spawn_mob_raw` at the parent's position,
  `child.extra.age = -24000`) — exactly vanilla's
  `child.setAge(-24000); child.moveTo(parent.x, parent.y, parent.z, 0,
  0); addFreshEntity`. No behaviour change. Rank 2: 334/348.

  Item 334 (report id 344, widely named: world/entity/ai/goal):
  carry-over. `OfferFlowerGoal.canUse`: 1.21.5's
  `Level.isBrightOutside()` body (`!hasFixedTime && skyDarken < 4`) is
  identical to 1.21.4's `Level.isDay()` body, and 1.21.5 removed
  `isDay()` entirely — a rename preserving the check. The Rust's
  `can_use` uses `cx.w.daylight`, which in the overworld is exactly
  `sky_darken() < 4` = `isDay()`. Recorded edge, not fixed: the Rust's
  `daylight` excludes the Nether outright and is true in the End
  (`darken` 0), while 1.21.4's `isDay()` is false in the End via
  `hasFixedTime` — that is the shared `daylight` field's model, a
  wider item, not this goal's. Rank 2: 335/348.

  Item 335 (report id 345, widely named: world/entity/animal/goat):
  ported. `GoatAi`: the 1.21.5 diff flips the mobGriefing branch in
  `RAM_TARGET_CONDITIONS` (an armor stand is a ram target only while
  mobGriefing is off in 1.21.4, only while it is on in 1.21.5 — a
  vanilla bug-fix). The Rust's `nearest_target` implemented neither
  the armor-stand part (moot: armor stands are not a `Species` and are
  not in the mob set, so no candidate can be one) nor the world-border
  part (real gap: vanilla requires the target's box within the
  border). Fixed: a `within_border` helper mirroring
  `WorldBorder.isWithinBounds(AABB)` (both corners, the max one nudged
  in by 1e-5, via the established `BorderGeometry::is_within_bounds`)
  now gates both the player branch and the mob branch of
  `nearest_target`; `MobWorld.border` is `None` when there is no
  border. New test pins the box check. Rank 2: 336/348.

  Item 336 (report id 346, widely named: world/entity/player):
  carry-over. `Abilities`: 1.21.5 adds `DEFAULT_*` constants and rewrites
  the NBT load to `getCompoundOrEmpty` / `getBooleanOr` / `getFloatOr`
  — the same values as 1.21.4's field defaults (flySpeed 0.05,
  walkSpeed 0.1, booleans false) and the same save keys
  (invulnerable, flying, instabuild, mayBuild, flySpeed, walkSpeed).
  The Rust's save writes flySpeed 0.05 / walkSpeed 0.1 / mayBuild 1
  when absent, and its load defaults match. No behaviour change.
  Rank 2: 337/348.

  Item 337 (report id 347, widely named: world/entity/variant):
  carry-over. `VariantUtils`: the class does not exist in 1.21.4 at all
  (the whole file is new in 1.21.5). Its methods only name behaviours
  1.21.4 already has: `Registry.getAny` exists in 1.21.4 and
  `MappedRegistry.getAny` returns `byId.getFirst()` — the Rust's
  "registry's first one" painting default (kebab, id 0) is exactly
  1.21.4's `Painting.defineSynchedData`; `selectVariantToSpawn` is
  emulated by the per-class `finalizeSpawn` paths (batch 79 fixed the
  save side, and the pig/cow/chicken registries are empty in 1.21.4).
  No `VariantUtils` reference in Rust code, only in comments naming
  the emulated method. Correctly absent for 1.21.4. Rank 2: 338/348.

  Item 338 (report id 348, widely named: world/food): carry-over.
  `FoodData`: 1.21.5 adds `DEFAULT_TICK_TIMER` /
  `DEFAULT_EXHAUSTION_LEVEL` and rewrites the NBT load to
  `getIntOr` / `getFloatOr` — the same values as 1.21.4's constructor
  (foodLevel 20, saturationLevel 5.0, exhaustionLevel 0, tickTimer 0).
  The Rust's `FoodData::default` is exactly (20, 5.0, 0.0, 0) and the
  player-save load defaults to the same four values when the keys are
  absent. No behaviour change. Rank 2: 339/348.

  Item 339 (report id 349, widely named: world/item/alchemy):
  carry-over. `Potions`: the diff is only `MobEffects` Java field
  renames (JUMP -> JUMP_BOOST, MOVEMENT_SPEED -> SPEED,
  MOVEMENT_SLOWDOWN -> SLOWNESS, DAMAGE_RESISTANCE -> RESISTANCE, HEAL
  -> INSTANT_HEALTH, HARM -> INSTANT_DAMAGE); the registry ids are
  unchanged and the potion list is identical (the one `ldc_w` -> `ldc`
  is a constant-pool encoding difference for "strong_poison", the same
  string). The Rust uses the registry ids as strings
  ("minecraft:jump_boost", "minecraft:resistance",
  "minecraft:instant_health", "minecraft:instant_damage") and never
  the Java field names — the `MOVEMENT_SPEED` hits are the attribute,
  which was not renamed. No behaviour change. Rank 2: 340/348.

  Item 340 (report id 350, widely named: world/item/crafting/display): carry-
  over. `SlotDisplay$ItemSlotDisplay`: 1.21.5 replaces
  `ByteBufCodecs.holderRegistry(Registries.ITEM)` with `Item.STREAM_CODEC`,
  but `Item.STREAM_CODEC` is built as exactly
  `ByteBufCodecs.holderRegistry(Registries.ITEM)` (1.21.5 `Item` static init)
  — a pure refactor to a named constant. The Rust's `slot_display`
  (lan/advancements.rs) writes the item as a varint holder id, matching both
  forms. No behaviour change. Rank 2: 344/348.

  Item 341 (report id 351, widely named: world/level/block/state/properties):
  ported. `BlockStateProperties`: 1.21.5 removes `ACTIVE` and adds
  `SEGMENT_AMOUNT`, `CREAKING_HEART_STATE`, `TEST_BLOCK_MODE`, `MAP`. The live
  defect is the creaking heart: 1.21.4's `CreakingHeartBlock` registers axis +
  `active` (bool) + `natural`, and its `updateState` only ever sets
  `active=true` when the heart has its logs; 1.21.5 replaces `active` with the
  `creaking_heart_state` enum (awake/dormant/uprooted) via
  `updateCreakingState`. The Rust had the whole 1.21.5 model:
  `heart_update_state` read the nonexistent property (`text()` returns '' for
  a missing key, so it never changed state — hearts never activated and no
  creaking ever spawned), the entity tick ran `updateCreakingState`, the
  comparator used the 1.21.5 distance-based signal, the animate gate checked
  for 'uprooted', and worldgen's `CreakingHeart` decorator errored on the
  unknown property (`d.with` returns `Err(InvalidState)`), so pale-oak-
  creaking trees placed no hearts. Fixed to the 1.21.4 model: new `active`
  prop key (`creaking_heart_state` key removed); `heart_update_state` is
  1.21.4's `updateState` (`active=true` when logs present, no night check);
  `server_tick_update` is 1.21.4's `serverTick` (no protector + no logs ->
  `active=false`; active + night + not peaceful + doMobSpawning + player
  within 32 -> spawn; protector removed at daybreak / too far / stuck, then
  `active=false` when logs are gone; unresolved protector -> nothing); the
  `getTicker` gate is `active` (not 'uprooted'); the comparator is
  `active?15:0` from the block state (`output_signal` field and
  `OutputChanged` event removed — 1.21.4 has no block-entity signal); the
  animate gate is `active=='true'`; the tear-down crumble uses
  ('active','true'); worldgen places the default state + `natural=true`. The
  rewritten level test (`creaking_heart_follows_its_logs`) pins `updateState`;
  the old test pinned the 1.21.5 form and could never have run (`w.id` panics
  on an unknown property with a fixture present; the level tests are skipped
  without prepared 1.21.4 resources, which the VM lacks). The other three
  properties are not defects: `SEGMENT_AMOUNT` belongs to 1.21.5's
  `SegmentableBlock` (leaf litter, absent from all 1.21.4 jars — the Rust's
  `LeafLitter` place arm is dead in 1.21.4, left in place); `MAP` is declared
  but used by no block in either version; `TEST_BLOCK_MODE` is the TestBlock
  dead support recorded in batch 73. Rank 2: 344/348.

  Item 342 (report id 352, widely named: world/level/chunk/status): carry-
  over. `ChunkStatus`: 1.21.5 only adds a `CODEC` field
  (`BuiltInRegistries.CHUNK_STATUS.byNameCodec()`) — a pure addition, no
  existing behaviour changed. The Rust has no chunk-status codec and needs
  none (chunk status names are never serialized by the Rust). No behaviour
  change. Rank 2: 344/348.

  Item 343 (report id 353, widely named: world/level/entity): carry-over.
  `PersistentEntitySectionManager`: 1.21.5 only adds `isTicking(ChunkPos)`
  (chunkVisibility lookup + `Visibility.isTicking()`) — a new accessor for the
  entity-ticking check. The Rust already has it: `ticket_levels.rs`
  `entity_ticking` (`DistanceManager.inEntityTickingRange`, level <= 31,
  `ENTITY_TICKING`) and `Game.entity_ticking_area`, used by the creature
  ticking; block ticking is the separate level-32 check (`is_ticking_chunk`).
  No behaviour change. Rank 2: 344/348.

  Item 344 (report id 354, widely named: world/level/levelgen/feature): carry-
  over. `Feature`: 1.21.5 only registers `FALLEN_TREE` (`FallenTreeFeature` +
  `FallenTreeConfiguration`) — fallen trees are a 1.21.5 addition. The Rust
  already has a `FallenTreeFeature` (worldgen/tree/fallen.rs, decoded from
  feature data in feature_graph.rs), but it is unreachable in 1.21.4: no
  1.21.4 jar entry or JSON mentions `fallen_tree` anywhere, so the decoder
  never fires for 1.21.4 worldgen. Dead in this tree, left in place (as the
  poplar leaf arms were in batch 79). No behaviour change. Rank 2: 348/348.

  Item 345 (report id 355, widely named: world/level/storage): carry-over.
  `LevelStorageSource$LevelStorageAccess`: 1.21.5 changes one call in
  `modifyLevelDataWithoutDatafix` from `CompoundTag.getCompound("Data")` to
  `getCompoundOrEmpty("Data")` — a rename preserving the lenient behaviour:
  1.21.4's `getCompound` returns a new empty compound when the key is absent
  (bytecode offsets 45-52) and throws only on a wrong type, and 1.21.5's
  `getCompoundOrEmpty` does the same via `Optional.orElseGet`. The Rust does
  not implement `modifyLevelDataWithoutDatafix` at all (a save-management
  utility, not gameplay), and its level.dat reads handle a missing "Data"
  explicitly via `Option` (save.rs). No behaviour change. Rank 2: 348/348.

  Item 346 (report id 356, widely named: world/phys): carry-over.
  `AABB$Builder`: the whole class is new in 1.21.5 (109 added lines, no 1.21.4
  file) — a builder accumulating min/max into an AABB. The Rust has no AABB
  builder anywhere and constructs AABBs via `Aabb::new`
  (horizon_core::geometry); there is no 1.21.4 behaviour to match. No
  behaviour change. Rank 2: 348/348.

  Item 347 (report id 357, widely named: world/ticks): carry-over.
  `TickPriority`: 1.21.5 only adds a `CODEC` field (`Codec.INT.xmap(...)`) — a
  pure addition, and the serialized form is unchanged: 1.21.4's
  `SavedTick.saveTick` already writes `priority.getValue()`, an int, and the
  1.21.5 codec maps that same int. The Rust's `TickPriority`
  (horizon_core::ticks) is the same int-valued enum (ExtremelyHigh=-3 ..
  ExtremelyLow=3, `by_value(i32)`) used in-memory by the tick scheduling. No
  behaviour change. Rank 2: 348/348.

  Rank-3 triage (2026-11-28, owner-authorized; triage only, no batch porting;
  full table in `RANK3-TRIAGE.md`). All 190 remaining items sorted: 173 CLOSE /
  15 ACCEPT-WITH-CAUSE / 2 DEFER (Realms). Batches 84-124 assigned (41 batches):
  G1 wired gameplay 84-92, G2 commands/protocol/launcher 93-96, G3 worldgen
  97-100, G4 long-tail sweep 101-107 (8/batch), G5 render 108-112, G6 big system
  113-114 (359 gametest, 360 packrat, one each), G7 rank-4 verification 115-124.
  Three corrections to the prepared brief, each jar-verified: (1) rank 4 is NOT
  confirm-absent — all 155 classes are PRESENT in the 1.21.4 jars (155/155);
  'New in <package>' means 1.21.5 moved/rewrote them (e.g. BakedModel is a
  6-method interface in 1.21.4 client/resources/model; 1.21.5 has no BakedModel
  class at all), so rank 4 flips ACCEPT -> CLOSE as a pre-refactor verification
  sweep. (2) The gametest framework EXISTS in 1.21.4 (53 classes/jar); only the
  test-block additions are absent, so 359 is CLOSE (the Rust's test_runs.rs
  implements the 1.21.5 /test form — it references clear/reset/run/verify, which
  javap shows absent from 1.21.4 TestCommand, and misses trackAndStartRunner,
  which 1.21.4 has) while 464/465 stay ACCEPT. (3) Registry entries 545/546 exist
  in 1.21.4: minecraft:potion (EntityType.POTION) was split into
  splash/lingering_potion in 1.21.5, and entity.wolf.howl (SoundEvents.WOLF_HOWL)
  is absent from 1.21.5 — both CLOSE, verify the Rust's registries carry the
  1.21.4 forms. Reliable signals: report `removed: True` means 1.21.5-only (all
  51 verified absent from both 1.21.4 jars); 101/110 named methods found live in
  their modules (9 misses flagged per item). DEFER stays Realms only (362+497:
  realms.rs 12,946 lines, 8/8 named live, Realms present in 1.21.4 jars, but
  only reachable with a signed-in Microsoft account — owner scope call).

  Batch 84 (2026-11-28, rank 3 G1: report ids 365-368). Rank 3 now 17/149
  (13 accepted + 4 closed), rank 5 2/2 (both accepted); total 377/548.
  New ledger `rank345-progress.json` seeded with the triage's 15 ACCEPT +
  2 DEFER entries. 365 PORTED (mace): rank-2 item 119 was wrong — it
  removed the 1.21.5 `can_destroy_blocks_in_creative` Tool gate, reading
  "creative always destroys", but 1.21.4 gates the same behaviour through
  `Item.canAttackBlock`, overridden by MaceItem, SwordItem and TridentItem
  to `!isCreative()` and called first by
  `ServerPlayerGameMode.destroyBlock`. The Rust now has
  `held_item_can_attack_block()` gating `break_block_as_player` (game.rs),
  the creative `destroy_block` comment corrected (hand.rs), and the
  creative_mining probe + oracle printing false for creative +
  sword/trident/mace rows. Other MaceItem diffs NO-CHANGE (javap vs both
  jars): `hurtEnemy` boolean->void is signature-only; `getAttackDamageBonus`
  is float arithmetic in 1.21.4 vs double in 1.21.5 (at most 1 float ulp
  against the Rust's f64-then-f32); `canSmashAttack`, knockback constants
  (radius 3.5, power 0.7, heavy x2 after a fall > 5) and the 4th Tool arg
  all match. 366 CARRY-OVER (brush): `BrushableBlockEntity` brush methods
  take Player in 1.21.4 vs LivingEntity in 1.21.5 (signature
  generalization, no 1.21.4 behaviour); the renderer gains a 7th Vec3
  camera parameter in 1.21.5 (API-only); `BrushItem.onUseTick` is
  byte-identical. 367 PORTED (creaking): 1.21.4's `updateShape` calls
  `updateState` directly; 1.21.5 schedules a delay-1 tick whose `tick`
  applies it, and 1.21.4 has no `CreakingHeartBlock.tick` at all. The Rust
  had the 1.21.5 form (shapes.rs scheduled the tick); now updateShape
  applies `heart_update_state` immediately and `heart_tick` + its dispatch
  are removed. `animateTick` verified in animate_tick.rs (night + active +
  1/16 + surrounded by logs -> idle creak); `getStateForPlacement` verified
  (axis + updateState, natural untouched). Level test
  `creaking_heart_follows_its_logs` passes locally with the immediate form.
  368 PORTED (end): two 1.21.5 leaks — `EnderEyeItem.useOn` destroyed the
  block at each portal cell before setting END_PORTAL (1.21.4 sets it
  directly; destroy call removed from `use_ender_eye`), and
  `EndPodiumFeature` routed its end_stone/air/portal sets through
  `dropPreviousAndSetBlock` (destroy-with-drops when different; 1.21.4's
  `place()` is 8 plain `setBlock` calls, so `replace()` is now a plain set
  and `same_blocks` is gone). Carry-over: EyeOfEnder tick tail is
  `setPosRaw` in 1.21.4 vs `setPos` in 1.21.5 (the Rust sets position
  only); save/load is a codec-vs-save reshape with identical NBT and the
  same default-item fallback; EndPortalFrameBlock shapes are
  value-identical (BASE + EYE boxes, 1.21.5 rebuilds them via `Block.column`).

  Batch 85 (2026-11-28, rank 3 G1: report ids 370, 371, 373, 375). Rank 3
  now 21/149 (13 accepted + 8 closed); total 381/548. 370 PORTED (frog):
  `LongJumpToRandomPos.canStillUse` asks `isInWaterOrBubble()` in 1.21.4,
  narrowed to `isInWater()` in 1.21.5 — a frog (or goat, same shared goal)
  standing in a bubble column also stops the search. `can_continue` now
  requires the feet block not be a bubble column. Everything else in 370
  is carry-over (PossibleJump Record-vs-subclass, pickCandidate arity,
  playSound signatures, FrogAi no-diff). 371 PORTED (item_use): the
  `minecraft:weapon` component does not exist in 1.21.4 (it arrives in
  1.21.5), so the Rust's `weapon_damage` was None for every item and
  `weapon_wear` never fired — no melee weapon ever lost durability on a
  hit. In 1.21.4 the durability comes from each item's own
  `postHurtEnemy` override: SwordItem, MaceItem and TridentItem all do
  `hurtAndBreak(1, attacker, MAINHAND)`; AxeItem has no such override, so
  axes take no hit wear. New `weapon_damage_1214(name)` fallback in
  items.rs (same idiom as the shield_1214 BlocksAttacks fallback):
  Some(1) for `*_sword`, `minecraft:mace` and `minecraft:trident`, with a
  unit test pinning the set. Trident `releaseUsing` verified carry-over
  (1.21.4 throw: hurtWithoutBreaking, spawn at 2.5 with the damaged
  stack, CREATIVE_ONLY pickup in creative else `removeItem`; 1.21.5's
  `consumeAndReturn(1)` is the same observable behaviour); ItemInHandRenderer
  verified carry-over (`renderItem`'s boolean leftHand parameter dropped
  in 1.21.5, `mulPose` widened Quaternionf->Quaternionfc, no transform
  changes). 373 PORTED (goals): sheep eat short grass, not the 1.21.5
  `edible_for_sheep` tag — the tag does not exist in any 1.21.4 jar, so
  the Rust's two tag checks in EatBlock could never fire and sheep never
  ate tufts; both are now `here == "minecraft:short_grass"`. The
  water-avoiding stroll also takes the land position in a bubble column
  (1.21.4 `isInWaterOrBubble`, 1.21.5 `isInWater`). FleeSun/RestrictSun
  verified carry-over (`isDay` vs `isBrightOutside` are byte-identical)
  and MeleeAttackGoal verified carry-over (spectator check order is
  equivalent). NOTE: the eat-goal tests pass vacuously on CI when
  prepared resources are missing — CI green does not prove the tufts
  half. 375 CARRY-OVER (redstone): pressure-plate shapes are
  value-identical (1.21.4 `Block.box`, 1.21.5 rebuilds them via
  `Block.column`: PRESSED box(1,0,1,15,0.5,15), unpressed box(1,0,1,15,1,15),
  TOUCH_AABB (0.0625,0,0.0625,0.9375,0.25,0.9375)); `checkPressed` unchanged;
  entityInside/onRemove/playSound diffs are API-only. The Rust's
  `plate_check_pressed` is called from the entity-inside hook and the
  scheduled tick, matching vanilla. CopperBulb `checkAndFlip` differs
  only by the playSound signature.

  Batch 86 (2026-11-28, rank 3 G1: report ids 376, 378, 382, 383). Rank 3
  now 25/149 (13 accepted + 12 closed); total 385/548. 376 CARRY-OVER
  (turtle): all 7 inner goal/navigation classes have identical signatures
  1.21.4 vs 1.21.5; the two TravelGoal diffs are unobservable in 1.21.4 —
  the null guard on travelPos cannot fire (TRAVEL_POS defaults to
  BlockPos.ZERO and start() always sets it before tick) and stop() leaving
  travelPos set is invisible because isStableDestination's flag test and
  null test yield the same amphibious rule. The Rust's remote_mob_data
  Turtle ids 18->has_egg, 19->laying are already 1.21.4-correct
  (declaration order HOME_POS=17, HAS_EGG=18, LAYING_EGG=19, TRAVEL_POS=20,
  GOING_HOME=21, TRAVELLING=22). 378 PORTED (frames): 1.21.4 has no
  painting_variant component, no typed entity_data component and no entity
  `data` field — HangingEntityItem.useOn and ArmorStandItem.useOn both run
  updateCustomEntityTag (CustomData.loadInto), not 1.21.5's
  createDefaultStackConfig/applyImplicitComponents. stack_configured_entity
  now merges custom_data at top level via data_cmd::merged, only when its
  `id` names the placed type (parseEntityType is null without an id);
  frames.rs no longer applies CUSTOM_NAME at construction (a renamed frame
  placed in vanilla 1.21.4 gets no name — it arrives only through the
  merge); paintings.rs of_stack reads the variant from custom_data, the
  pre-pick override is gone (the id-checked post-pick merge is
  authoritative), and the tooltip's random line now shows when creative OR
  custom_data is non-empty but undecodable — the deliberate asymmetry,
  jar-verified: the tooltip decodes the variant with NO id check, placement
  applies custom_data ONLY when its id matches. The renderer is carry-over
  (all four frame models already baked; BakedModel-vs-BlockStateModel is
  internal). Tests: the block_entity_item test rewritten to the 1.21.4 form
  (id match merges, no-id / wrong-id / entity_data ignored, no `data` key)
  and a new of_stack test pinning the custom_data source (the 1.21.5
  component names nothing). 382 PORTED (interactions): one log line —
  Interaction.readAdditionalSaveData is otherwise identical (width/height
  any-numeric floats, response boolean, entity-data ids 8/9/10 match), but
  1.21.4 logs a decode failure of an attack/interaction record where 1.21.5
  reads silently; from_tag now log::warn!s on a present-but-undecodable
  record. 383 CARRY-OVER (golems): the only two differing lines in
  CarvedPumpkinBlock are pure renames — Entity.moveTo is snapTo in 1.21.5
  (identical body) and Level.blockUpdated is updateNeighborsAt (the 1.21.5
  2-arg form adds the redstone-orientation rework; ServerLevel.blockUpdated
  is updateNeighborsAt when not debug).
  Batch 87 (2026-11-28, rank 3 G1: report ids 384, 385, 393, 394). Rank 3
  now 29/149 (13 accepted + 16 closed); total 389/548. 384 CARRY-OVER
  (respawn_anchor): 1.21.4 has no RespawnConfig record — ServerPlayer keeps
  respawnDimension/respawnPosition/respawnAngle/respawnForced separately —
  and useWithoutItem checks dim+pos differ, then calls the 5-arg
  setRespawnPosition(dim, pos, 0.0F, false, true); the Rust already does
  exactly this. Comment-only fix (the struct doc and the set call now cite
  the 1.21.4 form). 385 PORTED (goat): RamTarget.tick halves the knockback
  when the target blocks the mobAttack — getItemBlockingWith wants a
  BLOCK-animation item (a shield) used for at least 5 ticks, no
  #bypasses_shield, no piercing arrow, and the flattened goat-to-player
  direction against the player's horizontal view dot < 0; 1.21.5's
  applyItemBlocking (shield damage, attacker pushback) is deliberately not
  implemented. New PlayerView.blocking (using + blocks_attacks + delay, the
  shield_1214 values) and ram_tick multiplies the power by 0.5 when the
  player blocks in front; mob targets always take full force. Test pins half
  power in front, full power behind and unblocked. 393 PORTED
  (jigsaw_blocks): loadAdditional reads final_state with getString, so a
  missing key is "" in 1.21.4 (1.21.5 defaults to "minecraft:air"); the load
  default and its test now use "". The name/target/pool crash on a missing
  key is deliberately not replicated (the Rust keeps its EMPTY_ID fallback);
  joint and priorities already match. 394 PORTED (structure_blocks): strict
  is a 1.21.5 addition, removed end to end — the StructureBlock.strict field
  (struct/new/save/load), the edit screen's STRICT toggle and label, the
  SetStructureBlock packet's strict flag (bit 8), the /place template
  trailing literal, and load_settings' strict parameter (known_shape now
  always false, flags always 2; setKnownShape itself exists in 1.21.4 and
  the engine-level known_shape handling is untouched). The bounding-box
  gizmos already matched 1.21.4's StructureBlockRenderer (game-master or
  spectator gate, 96 view distance, SAVE-always / LOAD-if-showBoundingBox
  box, SAVE+showAir invisible blocks, the four air/void/barrier/light
  markers); only the citations were wrong — BlockEntityWithBoundingBoxRenderer
  is 1.21.5-only — re-anchored to StructureBlockRenderer.
  Batch 88 (2026-11-28, rank 3 G1: report ids 395, 397, 398, 399). Rank 3
  now 33/149 (13 accepted + 20 closed); total 393/548. 395 PORTED (camel):
  1.21.4 CamelModel has toggleInvisibleParts — saddle and bridle show only
  when saddled, reins only when saddled and ridden — while 1.21.5's model
  has createBodyMesh() and no toggle; camel::anim now sets those three
  visibilities (ridden = equine::is_vehicle) right after the head rotation,
  and a resource-free test pins hidden/visible in all three states. 397
  CARRY-OVER (trident): signatures identical; the only diffs are a no-op
  dealtDamage = false init and getBooleanOr(..., false) versus getBoolean,
  and the Rust's flag() already returns false on a missing key. 398 PORTED
  (books): the tooltip already matched WrittenBookItem.appendHoverText
  exactly (only the stations.rs comment cited the 1.21.5 location);
  saveChanges and the ClickEvent form carry over (SuggestCommand is
  1.21.5-only); resolveBookComponents is now done at view time — opening a
  book or reading it from a lectern resolves its selector/score pages via
  resolve_component, as ServerPlayer.openItemGui and
  LecternBlockEntity.resolveBook do — with the caveat that the stack keeps
  its unresolved pages and re-resolves on the next open (vanilla writes the
  resolved content back and marks it resolved). The new book test is
  #[ignore]d (needs prepared resources, like the lectern tests). 399 PORTED
  (growth): GrowingPlantHeadBlock.updateShape loses the 1.21.5-only
  look-ahead — a head under a head or body no longer converts when the
  neighbour against the growth changes (the Rust had the 1.21.5 branch from
  the parent form); to_body, the growth-direction conversion and the no-op
  super call already matched. ShearsItem.mineBlock is CARRY-OVER: the wear
  rule (1 durability on any non-fire block) already matches, and the boolean
  return is discarded by ServerPlayerGameMode.destroyBlock. The new kelp
  level test pins both updateShape branches but passes vacuously without
  prepared resources (locally and on CI, which has no prepare step).
  Batch 89 (2026-11-28, rank 3 G1: report ids 401, 402, 406, 407). Rank 3
  now 37/149 (13 accepted + 24 closed); total 397/548. 401 PORTED
  (phantom): 1.21.4's anchorPoint is never null — the ctor sets it to
  BlockPos.ZERO and a save without AX/AY/AZ leaves it there — while 1.21.5
  made it nullable and guards all three uses; AttackStrategy.stop now
  recomputes the anchor from the heightmap unconditionally, treating a
  missing anchor as the origin (anchor_above_target and select_next already
  matched: the first sets unconditionally, the second defaults a missing
  anchor to the phantom's own position). The new stop test pins both cases
  but passes vacuously without prepared resources (locally and on CI). 402
  PORTED (block_edits): BlockInput.place returns the set's result and loads
  the tag only on success — 1.21.5 also loads on failure and lets a changed
  entity flip the result to true, and skips the neighbour-shape
  normalization when flags & 16 (no command caller passes 16; both pass 2).
  cmd_setblock and cmd_fill now fail/count only on placed; the tag load is
  gated on placed in both. LevelChunk.setBlockState returning null when old
  == new is what makes the set fail on an unchanged state (verified in the
  jar). Two new #[ignore]d tests pin it: setblock/fill with the identical
  state and a new CustomName tag fail/count nothing and load nothing, while
  a changed state places and loads the tag. Caveat: fill still skips
  normalization for non-note blocks when the stored state already equals
  the input (place would re-normalize; only reachable with a state a
  non-normalizing path wrote). 406 CARRY-OVER (enchant_effects):
  PlaySoundEffect.apply differs only by the playSound Player->Entity
  widening, UnobstructedPredicate.test only by the VoxelShape.move overload
  (the same unit cube either way). 407 CARRY-OVER (inventory_slots):
  renderItem's isLeftHand/zOffset differences are dead on the GUI path
  (zOffset is always 0, never left-handed), and the chat hover dispatch
  already implements the 1.21.4 form exactly (text split at
  max(guiWidth/2, 200), item tooltip, entity only with advanced tooltips).
  Method note: norm.py strips method names, so an IDENTICAL verdict hides
  renames like setValue/trySetValue and the playSound Player->Entity
  widening — re-check any IDENTICAL verdict with names kept before closing
  an item on it.
  Batch 90 (2026-11-28, rank 3 G1: report ids 408, 409, 410, 413). Rank 3
  now 40/149 (13 accepted + 27 closed); total 400/548. 409 CARRY-OVER
  (sniffers): SnifferEggBlock.tick differs only by two playSound
  Player->Entity widenings and the moveTo/snapTo rename (hatchBoost is
  identical); the Rust's sniffer_egg_tick already plays crack/hatch at the
  position and hatch_sniffer spawns at the egg + 0.5. The SHAPE is identical
  too: 1.21.5's Block.column(14.0, 12.0, 0.0, 16.0) is box(1, 0, 2, 15, 16,
  14) by javap (8-7, 0, 8-6, 8+7, 16, 8+6), exactly 1.21.4's box call, and
  the Rust takes block shapes from prepared resources anyway. 410
  CARRY-OVER (wandering_trader): tick's int->void return is unused; the
  Rust already returns () with the jar's constants (1200/24000/25/75).
  413 CARRY-OVER (buckets): SolidBucketItem.emptyContents differs only by
  the Player->LivingEntity widening; the Rust's bucket_empty takes no user.
  408 DEFERRED for a protocol-layer decision (not a carry-over: the jar
  diff is real): replaceWithPacketData takes heightmaps as NBT in 1.21.4
  and as a Map in 1.21.5, but the adapter already bridges the 769 form
  (v1_21_4.rs::chunk NBT->map, v1_21_11 step inserting the fluid count;
  proven by nbt_heightmaps_and_prefixed_palettes_reach_the_pinned_chunk)
  and the native ChunkData/Section codec is the 26.2 internal layout
  (map heightmaps, unprefixed palettes, fluids i16), so a heightmaps-only
  NBT change would break the adapter's generic chunk path and fix nothing
  observable. What remains is the direct path: a real 1.21.4 server
  connects with no adapter (older_server returns None for the pinned
  version; ConnReader::recv returns the packet untranslated) and the
  game's decoder fails on raw 769 chunk bytes — proven by decoding a
  769-form packet with play::Clientbound::decode (ERR "NBT: NBT at byte
  1: unknown tag type"). The fix is either routing Current-version
  connections through the adapter or re-pinning the native chunk codec to
  769 with the adapter keeping its own 26.2-layout codecs; that decision
  also touches closed item 161's verdict that the adapter handles the
  prefixed palettes. No code change in this batch.
  Batch 91 (2026-11-28, rank 3 G1: report ids 414, 416, 417, 418). Rank 3
  now 44/149 (13 accepted + 31 closed); total 404/548. 414 CARRY-OVER
  (tree decorator): TreeFeature differs only by isVine private->public
  (body identical) and the new getLowestTrunkOrRootOfTree helper; the
  Rust's Lists::lowest() implements the helper exactly and 1.21.4's
  AlterGroundDecorator.place has the same logic inline (verified with
  names kept), so the AlterGround arm already matches 1.21.4.
  PlaceOnGroundDecorator does not exist in the 1.21.4 jar (javap: class
  not found) and no 1.21.4 data can reference minecraft:place_on_ground;
  the arm stays as dead-in-1.21.4 code per the FallenTreeFeature
  precedent. Comment-only change: lowest()'s doc now names the 1.21.4
  inline site and the variant is marked 1.21.5-only. 416 CARRY-OVER
  (wither skull): checkSpawn differs only by the WitherBoss.moveTo ->
  snapTo rename; the Rust's check_wither_spawn already spawns at the
  middle of the bottom row +0.5/+0.55/+0.5 with the forwards-axis yaw,
  make_invulnerable and the 50-block SUMMONED_ENTITY trigger. 417
  CARRY-OVER (fishing rod): use differs only by two playSound
  Player->Entity widenings; the Rust's use_fishing_rod already plays
  retrieve (1.0) and throw (0.5) at the player with pitch 0.4 /
  (rand*0.4 + 0.8). 418 CARRY-OVER (golem defense): canUse differs only
  by instanceof pattern-matching form (same isSpectator/isCreative
  calls); the Rust's DefendVillage::can_use already matches
  (spectator/creative early-false, 10x8x10 box, reputation <= -100,
  line of sight).
  Batch 92 (2026-11-28, rank 3 G1: report id 420). Rank 3 now 45/149
  (13 accepted + 32 closed); total 405/548. 420 PORT (pumpkin): the
  useItemOn diff itself is only the playSound Player->Entity widening,
  but the Rust carried the 1.21.11 form and rolled the carve/pumpkin
  loot table, which does not exist in 1.21.4 (arrived in 1.21.9) -- so
  carving dropped no seeds, the same live defect 1.21.7 had before its
  fix. The jar drops 4 pumpkin seeds directly in code (spawned at
  x+0.5+dirX*0.65, y+0.1, z+0.5+dirZ*0.65, velocity 0.05*dir +
  random.nextDouble()*0.02, 0.05 y). Fix mirrors the 1.21.7 fix
  (87f06930): drops = vec![ItemStack::new("minecraft:pumpkin_seeds",
  4)], keeping native_use_drops so mods can still add drops. Everything
  else already matched: shears check, Y-face player-direction-opposite
  facing, PUMPKIN_CARVE 1.0/1.0 at the block centre, set_golem_head,
  damage_held(1), SHEAR game event, shears-used stat,
  item_used_on_block trigger. No new test, same as the 1.21.7 fix.
