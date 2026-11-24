# Batch 11 — Rank-3 template items (177), 26.2 parity sweep

- **Batch:** 11 (rank-3 gaps) — checks + triage only, no fixes.
- **Date:** 2026-11-24
- **Scope:** 26.2's OWN 177 rank-3 ledger items (`pins/26.2/rank3-progress.json`,
  the `closed` list: buckets A 90 / B 64 / D 23; verdicts no-change 143 /
  ported 32 / accepted-gap 2), triaged against the named 26.2 jars and the
  26.2 port in `versions/26.2/crates/` (branch `parity-sweep-26.2`, synced
  to main 732c800f). Vanilla 26.2's own jar/data is the spec throughout;
  the 26.3 base's form was never trusted. **Scope difference from the
  1.21.7 Batch 11:** that batch triaged the 1.21.11 TEMPLATE's 247 rank-3
  items FOR 1.21.7; this batch verifies 26.2's own port ledger — i.e. each
  "closed" verdict is checked to actually hold in the current tree.
- **Ledger slice:** `batch-11.json` (same shape as
  `pins/1.21.7/sweep/batch-11.json`).

**Coordinator note (2026-11-24):** B11-G01 (MAJOR, beds) was
independently re-verified — the mapping, the five dead arms, the empty
`Class::Bed` dispatch, the absence of `AbstractBedBlock` from both 26.2
jars and its presence in the 26.3 client jar all check out. Tally
convention differs from 1.21.7 Batch 11: the 4 defects are cross-cutting
(no owning rank-3 item) and are NOT counted in the 177-item tally
(175 pass + 2 accepted-gap). `horizon-prepare` for 26.2 finished after
this batch ran (2048.7 s, EXIT 0); `block-behaviours.json` now exists,
so B11-H01's data precondition is met — the chain-liveness re-run
itself remains follow-up work.

## Tally (177 items)

| Verdict | Count |
|---|---|
| pass | 175 |
| gap (defect attached) | 0 |
| needs-new-harness | 0 |
| needs-in-game-probe | 0 |
| accepted gap (ledger verdict, §5) | 2 |

Defects found: **4** (`B11-G01`–`B11-G04`), of which 1 major, 3 minor.
All four are cross-cutting (class-table sweep + 1.21.7-defect cross-checks)
with no single owning rank-3 item — they are recorded as defects, not item
verdicts, so the defect count (4) exceeds the gap-item count (0).
Harness follow-ups: **1** (`B11-H01`, the class-table chain-liveness half,
blocked on prepare). No oracle defects, no probe items.

Headline: the 26.2 port's rank-3 ledger holds up well. Of the 23 defects
the 1.21.7 Batch 11 found, only the beds defect carries over to 26.2
(B11-G01) — the rest fail to reproduce because 26.2 is genuinely 26.x in
exactly the places 1.21.7 is not (26.x class names, 26.x tags, 26.x
behaviour forms; see §6.6). The port's own 32 "ported" verdicts all verify
in the current tree.

## §1 Defects

### B11-G01 — Beds place foot-only; sleep is unreachable (major; = 1.21.7 B11-G01)

`Class::AbstractBed = "AbstractBedBlock"` (classes.rs:62) never matches any
26.2 chain: 26.2 has no `AbstractBedBlock` — beds are `BedBlock` directly
(jar-verified: `BedBlock extends HorizontalDirectionalBlock`, no
`AbstractBedBlock` in either 26.2 jar). `Class::Bed = "BedBlock"`
(classes.rs:63) exists but NOTHING dispatches on it (grep-confirmed across
all crates). Every bed arm is dead:

- `place.rs:356` (placement check), `place.rs:1481` `set_placed_by` — the
  ONLY place a `"head"` part is ever set (place.rs:1483, grep-confirmed) —
  so a player-placed bed is a foot-only single block.
- `interact.rs:365` — the ONLY emitter of `PlayerEffect::Sleep`
  (interact.rs:377/380) — so clicking ANY bed (including village beds)
  does nothing; `try_sleep` never fires.
- `hooks.rs:459` (creative break), `shapes.rs:124` (neighbour update).

`BlockData::is` checks the full chain, so no fallback exists. Severity
major: sleep/spawn is core gameplay. **26.3-leak:** the 26.3 client jar
CONTAINS `AbstractBedBlock` (26.3 reintroduced the abstract bed class), so
the mapping is 26.3-correct and 26.2-wrong — the port carried the 26.3
class-table mapping into a version whose jar contradicts it. Note the
irony at interact.rs:366-369: the comment documents the 26.2 form
("26.2's BedBlock.useWithoutItem redirects a foot click to the head…
(26.3's AbstractBedBlock added the pair check)") while the arm itself
still dispatches on the dead 26.3 name. The fix is a 26.2-direct one
(dispatch on `Class::Bed`), not a re-base propagation.

### B11-G02 — Three number providers return 0 (minor, datapack-only; = 1.21.7 B11-G12)

Vanilla 26.2 has 6 number providers (`EnchantmentLevelProvider`,
`ScoreboardValue`, `StorageValue` all present in the server jar). Both Rust
implementations handle only 3 (`minecraft:constant`, `minecraft:uniform`,
`minecraft:binomial`) — `minecraft:enchantment_level`,
`minecraft:score`, `minecraft:storage` fall to `_ => 0`
(horizon-world loot.rs `int`/`float` 1346-1385; horizon-items items.rs
`number` 2033-2063). Data check: 0/1,355 vanilla 26.2 loot tables use the 3
missing providers — vanilla loot is unaffected. The only
`enchantment_level` uses in 26.2's datapack are the `chance` providers in
`enchantment/soul_speed.json` and `enchantment/thorns.json`, and those go
through the SEPARATE enchant.rs `random_chance` path, which handles
`enchantment_level` explicitly (enchant.rs:875-886:
`level_value(amount, level)`) — verified against vanilla 26.2's
`EnchantmentLevelProvider.getFloat`, which is `amount.calculate(level)`
with NO multiplication by level, exactly what the Rust computes. So the
gap is datapack-only: datapacks using the three providers in LOOT get 0.

### B11-G03 — Recipe-book fuel slot cycles fuels in the wrong order (minor; = 1.21.7 B11-G15)

Vanilla 26.2 `SlotDisplay$AnyFuel.resolve` returns
`FuelValues.fuelItems()` — the keySet of an insertion-ordered
`Object2IntLinkedOpenHashMap` populated by the HARDCODED
`FuelValues.vanillaBurnTimes` sequence (LAVA_BUCKET, COAL_BLOCK, BLAZE_ROD,
COAL, CHARCOAL, #LOGS, #BAMBOO_BLOCKS, #PLANKS, …; javap-verified). First
fuel shown: lava bucket. The Rust groups fuels by burn-time value in
registry order (recipe_book.rs:566-585, native_panel_recipes.rs:173-190) —
in registry order the first fuel is oak planks (protocol id 63; coal 924,
lava bucket 1042), so the first-shown fuel is a 300-tick plank, not lava
bucket. Burn TIMES unaffected — display order only. **26.3-form
carry-over:** 26.3's `AnyFuel.resolve` uses
`DataComponentLookup.findAll(COOKING_FUEL)` (javap-verified on the 26.3
client jar) — the grouping-by-component behaviour the Rust implements;
26.2's insertion-ordered FuelValues form is what the Rust should match.

### B11-G04 — F3 sound mood has one space before (Mood (minor, cosmetic; part of 1.21.7 B11-G20)

Vanilla 26.2's `DebugEntrySoundMood` appends
`String.format(ROOT, "  (Mood %d%%)", mood)` — TWO spaces — to
`Library.getChannelDebugString` (javap-verified on the 26.2 client jar).
The Rust's `sound_mood_line` (debug_overlay.rs:399-408) formats
`"Sounds: {}/{} + {}/{} (Mood {}%)"` — ONE space. The other two thirds of
1.21.7's B11-G20 do NOT apply to 26.2: vanilla 26.2's Local Difficulty
line has NO Day suffix (`"Local Difficulty: %.2f // %.2f"` — the day count
is a separate `DebugEntryDayCount` entry, `"Day #"` literal, which the Rust
renders as `"Day #{}"`, debug_lines.rs:55-57), and vanilla 26.2's memory
lines use MiB (`"Mem: %2d%% %03d/%03dMiB"`), as does the Rust. Cosmetic.
Note the one-space form matches NEITHER 26.2 nor 26.3 (26.3's entry
literal is also two-spaced) — it is a 26.1-form leftover in the base, not
a 26.3 leak.

## §2 Oracle repair

None. This batch ran no drivers and repaired no oracle. (The cauldron
oracle repair B11-O01 belongs to the 1.21.7 sweep, not this one.)

## §3 Needs-new-harness

### B11-H01 — Class-table chain-liveness half (follow-up for when prepare lands)

The §6.1 sweep did the jar-existence half: every `Class::X` mapping was
checked against both 26.2 jars (268/271 live, 3 dead). The 1.21.7 sweep's
second half — liveness against the `block-behaviours.json` chains (which
blocks' chains actually contain each mapped name) — cannot run for 26.2
yet: no prepared 26.2 `block-behaviours.json` exists (prepare pending; no
`.local/horizon-client/26.2/`). The jar half already settles the live-arm
question for the dead mappings (a name absent from the jar cannot appear
in any chain), but the converse — a jar-live name that no 26.2 block's
chain contains (e.g. an abstract class nothing extends, or a class whose
blocks are all 26.3-only) — is unmeasured.

**Harness design:** once horizon-prepare lands for 26.2, export
`block-behaviours.json` (1,196 blocks) and re-run the §6.1 sweep against
the chains: LIVE iff ANY alternative appears in ≥1 chain. Any mapping
live in the jar but dead in the chains gets its arms grepped and triaged
the same way B11-G01's were. The jar-side index is already built
(`server-classes.json` + `client-classes.json` in this batch's scratch).

## §4 Needs-in-game-probe

None. The 1.21.7 batch's probe item (F3 chart footer, item 519) does not
carry to 26.2: 26.2 HAS the DebugEntry system, and the chart gridlines
were ported in item 476 (verified uppercase). No rank-3 item needed a
probe to settle.

## §5 Accepted gaps / notes (not defects)

- **B11-A01 — Merged LAN screen (item 427, ledger accepted-gap).** The Rust
  ports the 26.3 merged WorldOptionsScreen (lan_screen.rs, incl.
  `open_multiplayer_options` at lan_screen.rs:712, reached from the pause
  menu's Multiplayer Options button) while 26.2 keeps WorldOptionsScreen
  and MultiplayerOptionsScreen separate. The underlying operations
  (publishServer, setWorldGameType, setWorldAllowCommands, difficulty,
  game rules, restrictions) all match 26.2; only the screen layout differs
  (one merged screen vs two separate screens, snapshot vs immediate apply
  on the world side). Accepted by the port ledger; splitting the screen is
  a multi-hour rewrite with LAN-flow regression risk. NOTE: the tracking
  note the ledger cites (screens.rs:455-458, "layout port outstanding")
  no longer exists in the tree — the gap persists but its in-code marker
  is gone.
- **B11-A02 — Edit-server save re-pings (item 436, ledger accepted-gap).**
  26.2's editServerCallback copies the edited entry back via copyFrom and
  KEEPS the old ping state; 26.3 resets to INITIAL (re-ping). The Rust's
  edit-save rebuilds the list fresh (`Self::list(saves)`,
  multiplayer.rs:1753/1767/2087), matching 26.3 — a 1-2s re-ping
  difference, deferred as a multiplayer-screen lifecycle refactor.

## §6 Systemic findings

### §6.1 The class table: 3 dead mappings, 1 live-arm rename

Method: parse every `Class::X = "Name" | "AltName"` in
`versions/26.2/crates/horizon-world/src/level/classes.rs` (271 mappings);
LIVE iff ANY alternative exists as a class (top-level or nested) in either
26.2 jar (server-game 7,445 classes; client 10,952). Jar-existence half
only — see B11-H01 for the chain-liveness half.

**Group A — RENAME: 26.2 content exists under another name; live arms
dead (1):** AbstractBed→BedBlock (B11-G01). The other eight 1.21.7 renames
do NOT carry to 26.2 — the 26.x names MATCH the 26.2 jar (verified each,
not assumed): FarmlandBlock ✓, SnowyBlock ✓, SpreadingSnowyBlock ✓,
NetherFungusBlock ✓, NetherRootsBlock ✓, Path ✓ via the dual name
(`"PathBlock" | "DirtPathBlock"`, classes.rs:211 — PathBlock itself is
absent, DirtPathBlock present), RedstoneWire ✓ via the dual name
(`"RedstoneWireBlock" | "RedStoneWireBlock"`, classes.rs:227 — the 26.3
name is absent, 26.2's RedStoneWireBlock present), Speleothem ✓
(`SpeleothemBlock` is the ABSTRACT parent of `PointedDripstoneBlock` in
26.2, javap-verified — the mapping is live through the chain). The
26.2 port already carries the dual names the 1.21.7 template had to add.

**Group B — NO 26.2 CONTENT: dead but harmless (2):** StrawBedBlock
(26.3-only; 1 arm: update.rs:429) and ShelfMushroomBlock (26.3-only; 4
arms: bonemeal.rs:94, place.rs:493, shapes.rs:137, survive.rs:42). Both
classes exist in the 26.3 jar and neither has any block in 26.2 data (0
straw_bed / shelf_mushroom entries in blocks.json). Arms unreachable.
Hygiene note, not gaps. (LilyPad is the reverse of 1.21.7: 26.2's table
maps the correct `LilyPadBlock`; `WaterlilyBlock` is absent from the 26.2
jars. CopperChestBlock, PotentSulfurBlock, SulfurSpikeBlock,
CopperGolemStatueBlock are all live in 26.2.)

Also verified: horizon-prepare's survival.rs:44 maps
`("RedStoneWireBlock", "RedstoneWire")` — the correct 26.2 capital-S name
(1.21.7's prepare mapped the lowercase name).

### §6.2 The 26.x tag-name pattern INVERTS for 26.2

The 1.21.7 sweep's systemic cause — the port reads 26.x tag names for tags
that don't exist in 1.21.7 data — does not apply to 26.2, because 26.2 HAS
the 26.x tags. Every tag read this batch checked is live in 26.2's data
(client-jar datapack, tag-file existence + contents):

- `minecraft:cauldron_can_remove_dye` (item tag: leather helmet /
  chestplate / leggings / boots, leather horse armor, wolf armor) — the
  Rust's wash gate (cauldron.rs:7-12, :214) is live; 1.21.7's B11-G23
  does not apply.
- `supports_chorus_plant` / `supports_chorus_flower` (both: end stone) —
  survive.rs:102/132/137 live; 1.21.7's B11-G21 does not apply.
- `overrides_mushroom_light_requirement` — survive.rs:266 live;
  `mushroom_grow_block` is the ABSENT one in 26.2 (the reverse of 1.21.7);
  1.21.7's B11-G17b does not apply.
- `cannot_support_snow_layer` / `support_override_snow_layer` —
  survive.rs:295/297 live; the 1.21.11 names
  (`snow_layer_can_survive_on` / `snow_layer_cannot_survive_on`) are
  absent in 26.2 (the reverse of 1.21.7); 1.21.7's B11-G16b does not apply.
- `speleothems` (pointed_dripstone + sulfur_spike) — live; the dripstone
  cauldron-fill tag gate works.
- `convertable_to_mud` — live (item 446's 26.2 spelling confirmed).

Correctly-absent tags, correctly handled: `sculk_growth_inhibitors` is
absent from 26.2 data and the Rust checks sculk_sensor/sculk_shrieker
blocks explicitly (item 423, verified); the potion tags `douses_fire` /
`hurts_water_sensitive_entities` / `extinguishes_entities` /
`rehydrates_axolotls` are absent (26.2's `PotionTags` has ONLY
`TRADEABLE`) and the Rust gates splash water on `Potions.WATER` equality
(splash_water.rs:42-51), matching vanilla 26.2's
`AbstractThrownPotion.onHitBlock` (bytecode 55-63:
`PotionContents.is(Potions.WATER)`); 1.21.7's B11-G13 does not apply.

### §6.3 26.3-form code that 26.2's jar contradicts

The 26.2 analogues of "template fixes not in the tree": the port carries
26.3 forms in exactly two places where 26.2's jar/data says otherwise —
the AbstractBed class mapping (B11-G01: `AbstractBedBlock` exists in the
26.3 jar, not in 26.2's) and the AnyFuel grouping (B11-G03: 26.3 resolves
via `DataComponentLookup.findAll(COOKING_FUEL)`; 26.2 via the
insertion-ordered `FuelValues`). Both are 26.2-direct fixes, not re-base
propagations. (The Mood spacing, B11-G04, is NOT 26.3-form — 26.3's entry
literal is two-spaced too; it is a 26.1-form leftover in the base.) The
StrawBed/ShelfMushroom mappings are 26.3-only content carried harmlessly
(§6.1 Group B).

### §6.4 Re-base WARNING: 26.2 is 26.x where 1.21.7 is not — do not propagate 1.21.7 forms down

The 1e-5 fluid-current floor: present in vanilla 26.2 (the constant
9.999999747378752E-6 appears 7x in 26.2's Entity class) AND in the Rust
(entity_fluid.rs:102, `length_squared() < 1.0e-5`). No floor-removal to
propagate; the 1.21.7 §6.4 warning (the template removed the floor for
1.21.11, but 1.21.7 HAS it) stands as a general per-version caution, and
26.2 needs no change. More broadly, this batch found NINE behaviours
where 26.2's vanilla form differs from 1.21.7's and the Rust already
matches 26.2 — a re-base must not propagate the 1.21.7 forms to 26.2:
Riptide passenger check (26.2 HAS `isPassenger` in `releaseUsing`,
offsets 61-66), SwellGoal alive check (26.2 HAS `isDeadOrDying` in
`canUse` + `tick`), polar-bear fox-goal age gate (26.2's selector is the
`!isBaby` lambda), lead-knot discard on failed tie (26.2 discards the
fresh knot, offsets 139-152), mushroom bonemeal gate (26.2 requires the
huge-mushroom feature + `4 + foliageRadius` height), sapling bonemeal
height check (26.2: `TreeGrower.getMinimumHeight` + build height, no
`canGrow`), trader heightmap (26.2 registers WANDERING_TRADER with
MOTION_BLOCKING_NO_LEAVES), trader storage (26.2 uses SavedData
`data/minecraft/wandering_trader.dat`, fields `spawn_delay` /
`spawn_chance`), shipwreck `heightAdjusted` caching (26.2 HAS the field).

### §6.5 Class-absence headline: 39 items name 26.3-only classes (59 tokens)

39/177 items name ≥1 class that exists in the 26.3 client jar (11,383
classes) but in NEITHER 26.2 jar. 59 distinct tokens, of which 2 are false
positives (`Removed` — an English word matching a 26.3 simple name;
`Stepped` — a fragment of `SteppedInterpolationHandler`), leaving 57 real
26.3-only classes: the removed systems (CommandResponseTracker,
ComputeCommand, ContextFloatProvider/IntProvider, SlotSourceArgument,
OitPipelineSet/OitRenderPassProvider/OitStage, TransientBlock,
ClientboundAddTransientBlockPacket, NarrationTrigger, ArgProvider,
ItemAccessor, ItemActivation, PalettedTextureManager, TrimTextureKey,
TrimOverride, RealmsHeader, InviteCode(+List/+RequestDto),
RealmTierConfigurationDto/RangeDto, LootContextSources, NoiseSpawnFinder,
SpawnTargetPoint, MoveResult, ClockInstance, Prediction,
PlayerRenderState, PostChainCache, PendingResults, EmissiveRisingParticle,
FallingParticlesLeavesBlock, AmbientLeavesBlockSoundPlayer,
ShelfMushroomBlock/Decorator, BlockMatcher/BoxBlockMatcher/
OrderedBlockMatcher/FilteredSectionCache/BlockScanUtils, AbstractOreFeature,
BlockReplacement, ArrayItemProvider, ComponentSlotCollection,
GrowableMutableContainer, BonemealSource, Linear/SteppedInterpolationHandler,
ContextFloat/IntProviderArgument, FeatureArgument, ItemQuads, Axes),
plus the rename `RedstoneWireBlock` (26.3's name; 26.2 has
RedStoneWireBlock). Each absent class was triaged individually:
pass-by-removal where the Rust deleted the surface (items 510, 566, 419,
501, 504, 426, 418, 421), pass-by-absence where the Rust never had the
surface (items 565, 585, 590, 537, 580, 522, 513, 553). 16 further
classlike tokens are absent from ALL THREE jars but are Rust-side names
(ArgType, BlockEditor, StructureEditor, TestBlockEditor, TrimRegistry,
ContentItem, EffectAction), JDK/Brigadier/Log4j classes (StringBuilder,
CommandSyntaxException, LoggingEventBuilder), or generic words — 0 real
vanilla absences beyond the 26.3-only set.

### §6.6 1.21.7 Batch 11 defects checked and WITHDRAWN for 26.2

Every 1.21.7 Batch 11 defect was re-checked against 26.2's jar/data/Rust.
Withdrawn (verified not-applicable, with the deciding evidence):

- G02 redstone wire — dual name live via RedStoneWireBlock (§6.1).
- G03 farmland/paths — FarmlandBlock + DirtPathBlock (dual) live (§6.1).
- G04 snowy — SnowyBlock/SpreadingSnowyBlock live (§6.1).
- G05 fungus/roots — NetherFungusBlock/NetherRootsBlock live (§6.1).
- G06 speleothem — SpeleothemBlock is PointedDripstoneBlock's parent (§6.1).
- G07 sapling bonemeal — 26.2's gate IS the height check; Rust matches.
- G08 riptide riding — 26.2 vanilla HAS the isPassenger check.
- G09 lead knot — 26.2 vanilla discards the knot on a failed tie.
- G10 creeper defuse — 26.2 vanilla HAS isDeadOrDying in SwellGoal.
- G11 polar-bear cubs — 26.2's fox-goal selector IS !isBaby.
- G12 number providers — APPLIES (B11-G02 above), datapack-only.
- G13 splash water — Rust uses Potions.WATER equality; 26.2 vanilla gates
  on it; no potion tags in 26.2 data (§6.2).
- G14 player-data layout — 26.2 vanilla IS players/data/ (LevelResource).
- G15 fuel order — APPLIES (B11-G03 above).
- G16 golem stroll + snow tags — stroll roll order matches 26.2 exactly
  (0.3 first, 0.7 second); snow tags are the 26.x names, present (§6.2).
- G17 mushroom — 26.2's gate genuinely requires the feature + height;
  Rust matches; light tag present (§6.2).
- G18 social gate — 26.2's hide/show buttons are NOT chat-gated (only the
  report button's tooltip uses chatReportable); Rust matches (report
  button disabled by design, no Mojang services).
- G19 trader — heightmap MOTION_BLOCKING_NO_LEAVES matches; storage is
  SavedData data/minecraft/wandering_trader.dat, matches (§6.4).
- G20 F3 lines — Day count and MiB MATCH 26.2; only the Mood spacing
  applies (B11-G04 above).
- G21 chorus — tags present in 26.2 data (§6.2).
- G22 dialog tooltip — 26.2's ItemDisplayWidget uses isHovered() only;
  Rust matches.
- G23 cauldron wash — tag present with leather armour (§6.2).
- H01 shipwreck — 26.2 HAS heightAdjusted; Rust caching matches.
- G01 beds — APPLIES (B11-G01 above). The only major carry-over.

## §7 Method notes

- **Item source:** 26.2's own rank-3 ledger,
  `pins/26.2/rank3-progress.json` (177 items in the `closed` list; counts
  verified: A 90 / B 64 / D 23; no-change 143 / ported 32 / accepted-gap
  2). `pins/26.2/work-items.json` (the full 653-item ledger) was consulted
  for context. The triage script + output referenced in
  rank3-progress.json (`~/workspace/mcir-work/26.2-behaviour/rank3-triage.{py,json}`)
  does NOT exist — the `26.2-behaviour` workdir is gone — so the
  method-intersection triage (A 90 / B 63 / D 24 in the ledger's `triage`
  string; note the B/D split differs by one from the item list) could not
  be re-read; triage here rests on the per-item evidence + jar/Rust
  verification instead.
- **Jar evidence:** named 26.2 server-game jar
  (`26.1-adapters/versions/26.2/jars/server-game.jar`, 7,445 classes,
  Mojang-mapped) + 26.2 client jar (`leakage-audit/jars/26.2-client.jar`,
  10,952 classes, a game jar). The 26.2 server jar in leakage-audit is a
  Mojang BUNDLER jar and was never javap'd. The 26.3 client jar
  (`jars-26.3/client-26.3.jar`, 11,383 classes) was used ONLY for the §6.5
  absence comparison and the §6.3 26.3-form checks — never as the spec.
  javap via JDK 25 (`~/workspace/tools/jdk-25/bin/javap`); each call took
  8–35s under load, batched 1–3 classes per exec. Class-existence checks
  used zipfile indexes of the jars (in scratch:
  `server-classes.json`, `client-classes.json`,
  `client26.3-classes.json`), not per-class javap.
- **Crosscheck:** automated class cross-check (`rank3-crosscheck.json` in
  scratch): every named vanilla class checked against both 26.2 jars and
  the 26.3 jar (see §6.5); every named Rust module checked against the
  26.2 tree. 4 named modules do not exist: 510 `compute.rs` and 566
  `gfx/oit.rs` were DELETED by their ports (expected); 591 `game/tabs.rs`
  never existed at that path (MenuTabBar is cited in comments in
  stats_screen.rs/realms.rs/title.rs); 592's module is
  `horizon-client/src/toasts.rs`, not `game/toasts.rs`. 3 items name a
  "widely named" module (463, 505, 470), not a file.
- **Family-pass discipline:** a pass rests on jar evidence (named classes
  exist in the 26.2 jar, or are correctly identified as 26.3-only) + the
  Rust file existing with the same logic. Sample-verified across
  patterns: all 32 "ported" verdicts were individually re-verified in the
  current tree (greps + line numbers in §8), plus spot checks on
  no-change items with specific jar claims (466 sapling gate, 461 BedRule,
  491 hurt_marked, 507 illusioner formula, 511 warden tracker, 529
  is_fuel, 563 deflect forms, 446 mud tag + wax sound, 490 phantom
  hurtTime, 524 SoulParticle, 553 narrator, 585 spawn finder, 590
  interpolation, 594 Prediction). The 1.21.7-defect cross-checks (§6.6)
  are the deep-verification layer: 23 defects re-checked with javap
  against 26.2's jar.
- **ExecuteCommand report hole (item 419's NOTE) — CLOSED.** The port
  ledger noted ExecuteCommand was absent from the 26.3→26.2 diff report
  and its non-slot differences were "covered by NO item". This batch
  compared the class between the 26.2 server-game jar and the 26.3 client
  jar method-by-method (javap -c, comment/constant-pool normalised,
  bytecode-hashed): 112 vs 114 methods; the only real differences are the
  slots/items conditional swap (countItems vs countSlots +
  SlotSourceArgument vs SlotsArgument — ported in item 419) and the
  ArgProvider/DataProvider rename (item 533). The 7 apparent shared-method
  diffs were whitespace artifacts of the normalisation (verified by
  diffing the bodies). No unported ExecuteCommand difference remains.
- **Data checks:** tag files + contents from the 26.2 client-jar datapack
  (811 tag files); block/item registries from the 26.2 datagen reports
  (`26.2/versions/26.2/datagen/reports/`, md5-verified distinct from
  26.3's dump); loot-table provider usage counted over all 1,355 vanilla
  loot tables + all datapack JSON.
- **No cargo builds, no driver runs, no git commits.** Static checks only
  (javap + python + grep), per the batch constraints; no `.local` data was
  needed or used. No file under `versions/26.2/` was modified.
- **Re-verification caveat:** all Rust greps ran against the sweep
  worktree (`~/workspace/mcir-work/sweep-26.2/versions/26.2/`, branch
  parity-sweep-26.2); line numbers are from that tree.

## §8 Per-item triage table

Verdicts: **pass** (verified against the 26.2 jar/data + Rust),
**accepted-gap** (the port ledger's own verdict, §5). No item verdicts
gap/harness/probe in this batch; the four defects are cross-cutting (§1).
"Family" passes rest on: the port found NO behaviour difference between
26.3 and 26.2 for the item (pure renames/re-shapes/codecs/bytecode
artifacts); the named classes exist in the 26.2 jar (crosscheck) or are
correctly 26.3-only; the Rust file exists with the same logic.
Sample-verified across patterns (see §7).

| Item | Verdict | Evidence |
|---:|---|---|
| 418 | pass | Ported, verified: JoinCode/PDP/tier/invite-code code gone (0 refs), 23 regions, AddRealmPopupScreen present, saveSettings unconditional. |
| 419 | pass | Ported, verified: /item tree has replace/modify only (tree_misc.rs:168/184), `if slots` gone, slot_source registry absent + slot_source_type 6 types in registries.json; ExecuteCommand bytecode comparison closes the report hole (§7). |
| 420 | pass | Ported, verified: Std140 vec3/ivec3 → (16,16) (post_chain.rs:64); Globals 56-byte layout (post_chain.rs:517). |
| 421 | pass | Ported, verified: invite-code service + tier config gone; no Client-Ref header, X-NetworkProtocolVersion only (client.rs:1080); template link/trailer plain strings. |
| 422 | pass | Family: authlib package move + presence try/catch only; Rust friends stub has no network call. |
| 423 | pass | Ported, verified: can_place_growth counts sensor/shrieker explicitly, false at 3 (level/sculk.rs:607, worldgen/sculk.rs:587); sculk_growth_inhibitors absent from 26.2 data. |
| 424 | pass | Rust already single-level (test_runs.rs:212: "26.2 has no dimension on a test"); 26.3 cross-dimension finder unported. |
| 425 | pass | Family: Builder vs Accessor param rename, identical bodies; FORTRESS_ENEMIES values same, no Rust surface. |
| 426 | pass | Ported, verified: transient blocks inert — no add-transient-block packet in the 26.2 jar; feeder dead by design (transient_blocks.rs:14). |
| 427 | accepted-gap | B11-A01 — merged LAN screen kept (26.3 form); ops match 26.2, layout differs. Ledger's tracking note (screens.rs:455) no longer in tree. |
| 428 | pass | Family: all 12 options-screen classes decompile identically; diffs are bytecode artifacts. |
| 429 | pass | Ported, verified: already-errors removed — /time acts unconditionally (commands.rs:2274-2323); MoveResult gone. |
| 430 | pass | Family: record-vs-class re-shape; place bodies identical modulo config param. |
| 431 | pass | Ported, verified: close_advancements_to_game added + wired to KeyL (advancements_screen.rs:183, play_keys.rs:189). |
| 432 | pass | Ported, verified: poplar removed (only a comment remains, animate_tick.rs:41); falling-leaves chances match 26.2's Blocks registration. |
| 433 | pass | Ported, verified: RegistryFileCodec error is 26.2's bare form (codec.rs:316: "Failed to get element {id}"); RangeSlotSource requires source. |
| 434 | pass | Family: GUI render classes decompile identically; Lighting differs only by the RenderPearl→Blaze3D import move. |
| 435 | pass | Family: GPU plumbing only; Rust cites vanilla CommandEncoder in a comment. |
| 436 | accepted-gap | B11-A02 — edit-server save rebuilds the list fresh (multiplayer.rs:1753/1767/2087), re-pinging like 26.3; 26.2 keeps ping state. |
| 437 | pass | Ported, verified: give/clear/enchant branch on target count (effects.rs:92/199/240); Tracker struct gone. |
| 438 | pass | Family: totem animation math byte-identical; block-overlay submission unported against both versions. |
| 439 | pass | Ported, verified: /place feature is ResKey(configured_feature) (tree_world.rs:65); inline machinery + tests/harnesses removed. |
| 440 | pass | Family: Builder vs Accessor param rename only, same as 425. |
| 441 | pass | Family: codec + setBlockAndUpdate vs setBlock(...,3) + unported Prediction flag; Rust use_beehive matches both. |
| 442 | pass | Family: UBO field order differs, values identical; Rust writes its own uniform layout (corrected by item 420 for Globals). |
| 443 | pass | Family: record vs class + FeaturePlaceContext re-shape; place bodies identical. |
| 444 | pass | Family: setBlockAndUpdate vs setBlock(...,3); renderer rotateDegrees vs mulPose, same angles. |
| 445 | pass | Ported, verified: default game type for owner, @Hz suffix dropped, Terrain Rendering + Window lines removed from live code (two test fixtures still carry the string), chart labels uppercase, debugGuiScale option removed. |
| 446 | pass | Family: codec + setBlockAndUpdate; mud tag already the 26.2 spelling convertable_to_mud (dispenser.rs:2241); wax sound 3003-only (dispenser.rs:2225). |
| 447 | pass | Family: DeviceInfo byte-identical apart from the package move. |
| 448 | pass | Family: glint math identical; shade boolean (26.2) vs shadeDirectionOverride (26.3) both parsed, 26.2 assets carry only the boolean. |
| 449 | pass | Family: quaternion vs addLocalCoordinates are the same rotation; syncVelocity diff is unmodelled networking. |
| 450 | pass | Book click already left-button (26.2 form); page keys same in both. |
| 451 | pass | Ported, verified: scoreboard/team counts branch on target count; regression test twenty_six_two_counts_targets_not_successes (scoreboard_cmd_tests.rs:333). |
| 452 | pass | Family: GpuFormat enum churn; Rust cites RGBA8_UNORM in a comment only. |
| 453 | pass | Family: 26.3 adds compileTaskStartTimeNs (unobservable counter); empty-mesh install is 26.3-only, Rust installs the compiled mesh as 26.2 does. |
| 454 | pass | Family: Builder vs Accessor rename + setBlockAndUpdate vs setBlock(...,3). |
| 455 | pass | Family: codec + setBlockAndUpdate; piercing swing reset happens via Player.onAttack in both. |
| 456 | pass | Family: shortcutKey vs key codes (same keys); needsValidCursorPos feeds only the IME caret report. |
| 457 | pass | Family: ArgProvider/DataProvider re-shape, identical semantics; /data modify compute unported (ComputeCommand absent from 26.2 jar). |
| 458 | pass | Family: TitleCommand Tracker tracks every target in 26.3 = 26.2's manual branching; Rust branches on target count; NbtContents limit has no Rust counterpart (resolves all, as 26.2 does). |
| 459 | pass | Family: rotate vs mulPose same transform; linesTranslucent depth state same; Rust debug_lines uses depth_write=false. |
| 460 | pass | Family: AbstractOreFeature extraction; bodies identical; Rust rule.test 2-arg form matches 26.2. |
| 461 | pass | BedRule already ported: dimension_type.rs reads 26.2's explodes field; destroy_on_leave documented absent. |
| 462 | pass | Family: record re-shape; replaceable predicate identical (lambda vs Feature.isReplaceable). |
| 463 | pass | Family: BlendFunction record re-shape; every Rust mention is a doc citation. |
| 464 | pass | Family: record re-shape; BonusChestFeature place body identical. |
| 465 | pass | TestDef has no dimension field — already the 26.2 form (26.3 adds one). |
| 466 | pass | Sapling already ported: isValidBonemealTarget is the height check only (javap-verified: TreeGrower.getMinimumHeight + isInsideBuildHeight); canGrow + BonemealSource are 26.3-only; GROWERS covers all 10 vanilla saplings. |
| 467 | pass | Family: fade math identical (elapsed >= fade ? 1 : elapsed/fade); Rust section_visibility implements it. |
| 468 | pass | Family: GPU/entity-section internals; Rust cites getStats/count in comments only. |
| 469 | pass | Ported, verified: unconditional place_netherrack_or_magma for drip columns (ruined_portal.rs:293), checked form kept for spread. |
| 470 | pass | Ported, verified: /tag branches on target count, returns changed count (data.rs:1403-1438); list-empty branches on target count too. |
| 471 | pass | Family: render-pass API only; CubeMapTexture.doLoad identical; Rust renders panoramas via wgpu. |
| 472 | pass | Family: digit-read form only; same floorMod formula, Rust tab_key implements (digit+9)%10. |
| 473 | pass | Family: data-driven (Rust reads preset biome tables + spawn_target ParameterPoints from 26.2 data); deepDarkRegion condition same. |
| 474 | pass | Family: Builder vs Accessor param rename only. |
| 475 | pass | Family: shouldRender identical (unused partialTicks param); beam rotateDegrees vs mulPose, same angles. |
| 476 | pass | Ported, verified: chart gridlines uppercase ("30 FPS", "60 FPS", debug_chart.rs:372-373). |
| 477 | pass | Family: Tracker tracks every player in 26.3 = 26.2's manual form; Rust cmd_transfer matches. |
| 478 | pass | Family: hasEnoughSpace iteration style only, same 2x3x2 collision check. |
| 479 | pass | Family: drag button 1 (26.3) vs 0 (26.2); Rust has no generic container drag state and per-screen handlers are left-button. |
| 480 | pass | Family: codec + one extra pos arg to the loot call; carve/seeds/sound/damage/stat identical. |
| 481 | pass | Ported, verified: frog land search is a block test (frog.rs:346: block below is minecraft:water), not a fluid test. |
| 482 | pass | Family: codec + setBlockAndUpdate vs setBlock(...,3). |
| 483 | pass | Ported, verified: dragon-egg teleport has the world-border check (interact.rs:598: within_world_border); event-2015 form already ported. |
| 484 | pass | Family: Builder vs Accessor rename; NetherFossilStructure decompiles identically. |
| 485 | pass | Ported, verified: trims are the 26.2 atlas form (TrimRegistry reads asset_name + override_armor_assets); 26.3 names appear only in doc comments. |
| 486 | pass | Family: LocatorBar decompiles identically; diffs are bytecode artifacts. |
| 487 | pass | Family: ApplyBonusCount condition form + getOptional rename only; bonus formulas byte-identical. |
| 488 | pass | Family: CommandSourceStack ctor arg list only; source itself identical. |
| 489 | pass | Family: rotateDegrees vs mulPose only, same angles. |
| 490 | pass | Rust uses hurtTime > 0 (phantom.rs:286), exactly 26.3's wasHurtRecently(). |
| 491 | pass | Already ported: TradeWithPlayerGoal gates on !hurt_marked (trader.rs:200) with the 26.2 hurtMarked comment + test. |
| 492 | pass | Family: log call only (exception object vs its message). |
| 493 | pass | Already ported: narration takes the hovered row unconditionally (26.2 form); 26.3's trigger gate noted in comment. |
| 494 | pass | Family: NowPlayingToast decompiles identically; diff is a bytecode artifact. |
| 495 | pass | Ported, verified: ARGUMENT_TYPES is the 26.2 list (57 entries, commands.rs:25; registries.json confirms 57); five 26.3-only types removed. |
| 496 | pass | Family: setBlockAndUpdate vs setBlock(...,3) only. |
| 497 | pass | Family: setBlockAndUpdate vs setBlock(...,3) only. |
| 498 | pass | Family: record vs class re-shape; IcebergFeature place body identical. |
| 499 | pass | Family: setBlockAndUpdate vs setBlock(...,3) only. |
| 500 | pass | Family: TrunkPlacer config re-shape; place bodies identical. |
| 501 | pass | Ported, verified: context-provider decode arms removed; ResourceOrId registries (predicate, dialog, item_modifier, loot_table) all exist in 26.2. |
| 502 | pass | Family: RenderPearl→Blaze3D imports + setUniform→bindTexture renames; close() byte-identical. |
| 503 | pass | Family: config-class re-shape; place bodies identical; Rust references are comments. |
| 504 | pass | Ported, verified: slot-source codec is 26.2's withAlternative form (codec.rs:89/212: single element or list, no tags); slot_source registry absent from 26.2 data. |
| 505 | pass | Family: SectionMesh record re-shape + a timing getter removed; no Rust surface. |
| 506 | pass | Family: pre-split quads vs prepareQuadList — same quads submitted, same animated condition; Rust references are doc comments. |
| 507 | pass | Illusioner uncertainty is a pure extraction (14 - difficulty*4); Rust uses it (illager.rs:666). |
| 508 | pass | RedstoneWireBlock→RedStoneWireBlock rename; 26.2's name is RedStoneWireBlock (jar-verified) and the class table carries both names (dual mapping, live). |
| 509 | pass | Family: section-emptiness methods; Rust cites ClientChunkCache in a doc comment only. |
| 510 | pass | Ported, verified: /compute removed — compute.rs gone, no dispatch arm; ComputeCommand/ContextFloatProvider/ContextIntProvider absent from 26.2 jars. |
| 511 | pass | cmd_warden_spawn_tracker is single-player-only (always the .single message, sculk.rs:328-337); Optional-vs-direct tracker unobservable. |
| 512 | pass | Family: getBoundingBoxForCulling signature re-shape; Rust mentions the renderer in a comment only. |
| 513 | pass | Family: NarrationTrigger field/ctor/accessor are 26.3-only; nest() identical modulo the dropped arg (already applied in item 129). |
| 514 | pass | Family: openMetadata/openResources renamed openFull/openPrimary (internal API); listResources/getResource unchanged. |
| 515 | pass | Rust draws centered text + narratable region — the 26.2 form; 26.3's FocusableTextWidget focus behaviour not present. |
| 516 | pass | Family: trigger-matching logic not ported; effects.rs is the HUD display, cites the trigger in one comment. |
| 517 | pass | Family: JigsawBlock codec + canAttach; no can_attach in the Rust; Pools.bootstrap has no Rust surface. |
| 518 | pass | Ported, verified: tick failure arms call command_failure and return Ok(1)/Ok(0)/Ok(0) (misc.rs:150/180/189). |
| 519 | pass | Family: GameModeSwitcherScreen/Slot/Icon decompile byte-identically; "changed" flags are artifacts. |
| 520 | pass | Family: listResourceStacks/listResources signature re-shape; Rust cites createReload in a comment only. |
| 521 | pass | Family: codec decode form only (holderSet vs homogeneousList, same JSON); Rust loads archetype items from tags. |
| 522 | pass | Family: world-upgrade notifications are 26.3-only (removed in 26.2); Rust cites broadcastNotification in a comment only. |
| 523 | pass | Family: simpleCodecs in 26.2 (no Rust surface); getStairsShape/scanForWater/tryScheduleDieTick unchanged. |
| 524 | pass | SoulParticle form verified: RisingParticle motion, scale 1.5, Block15 for SculkSoul only (particle_type.rs:206-207); EmissiveRisingParticle is 26.3's rename. |
| 525 | pass | Family: ResourceArgument accessors are 26.2-only additions; Rust calls none of them. |
| 526 | pass | Family: simpleCodec in 26.2 (no Rust surface); no behaviour method changed. |
| 527 | pass | Family: colour/count constants hoisted in 26.3, passed literally in 26.2 (0.45/0.45/0.5); quad bodies + random draws identical. |
| 528 | pass | Family: simpleCodec in 26.2; Rust cites the XP ranges in a comment. |
| 529 | pass | is_fuel uses the FuelValues burn_time table (menu.rs:1439); CraftingTableBlock only gains a codec. |
| 530 | pass | Family: null 8th arg to submitModel (signature); FenceBlock only gains a codec; GPU plumbing. |
| 531 | pass | Family: predicate-type change (Holder vs ContextAwarePredicate); matches() is the same three-predicate conjunction, evaluated via adv::conditions. |
| 532 | pass | Family: extractIcon decompiles identically; "changed" flags are artifacts. |
| 533 | pass | ArgProvider is 26.3's rename of DataProvider; tree identical; Rust providers() is the DataProvider.wrap equivalent. |
| 534 | pass | Family: simpleCodec in 26.2; suspiciousStewEffects/makeEffectList unchanged. |
| 535 | pass | Family: simpleCodec in 26.2; WeatheringCopperChestBlock.randomTick unchanged. |
| 536 | pass | Family: setBlockAndUpdate IS setBlock(pos, state, 3) in both versions (item 414); toggle/burnout identical. |
| 537 | pass | TreeDecorator helpers are 26.3-only; the only caller (ShelfMushroomDecorator) is 26.3-only (ShelfMushroomBlock absent from 26.2 jar + data). |
| 538 | pass | Family: Blaze3D.openUri → Util.getPlatform().openUri (the URI-opener move); same links, same screens. |
| 539 | pass | Family: extractBackground/extractEffects decompile identically; "changed" flags are artifacts. |
| 540 | pass | Family: simpleCodec in 26.2; no behaviour method changed. |
| 541 | pass | Family: DebugEntryCategory/DebugScreenEntries decompile identically; <clinit> flags are artifacts. |
| 542 | pass | Ported, verified: is_pause_screen — StructureEditor + TestBlockEditor override to false (block_editor.rs:141/179-180); Jigsaw/TestInstance keep the default true. |
| 543 | pass | Family: packet.tags() vs packet.getTags() accessor rename; canUseCommandBlocks is 26.3-only. |
| 544 | pass | Ported, verified: raid feedback literal has the trailing space (raid.rs:52: "Found a started raid! "). |
| 545 | pass | Family: ChunkMap$TrackedEntity decompiles identically; <init> flag is an artifact. |
| 546 | pass | Family: static vs instance form; same values, same transform order (translate 0.5, rotate Y, translate offset, scale 0.25). |
| 547 | pass | Family: TutorialToast/Icons decompile identically; extractRenderState flags are artifacts. |
| 548 | pass | Family: setBlockAndUpdate vs setBlock(...,3) (item 414). |
| 549 | pass | Family: simpleCodec in 26.2; MAX_AGE + growth unchanged. |
| 550 | pass | Family: extractContent/extractIcon decompile identically; "changed" flags are artifacts. |
| 551 | pass | Ported, verified: cmd_unpublish failure arms call command_failure + Ok(0) (extra.rs:880-897). |
| 552 | pass | Family: 26.3 validates atlas materials; 26.2 takes materials.get directly; Rust performs no atlas validation — already the 26.2 form. |
| 553 | pass | Narrator already 26.2-form: no NarrationTrigger anywhere (screen.rs:137); schedule takes (delay, ignore_suppression) only. |
| 554 | pass | Family: simpleCodec in 26.2; ChorusPlantBlock.getStateWithConnections unchanged. |
| 555 | pass | Family: registry rename (FEATURE vs CONFIGURED_FEATURE); place call identical. |
| 556 | pass | can_place_growth already implements 26.2's direct loop (worldgen/sculk.rs:587; level/sculk.rs:607); blockscan package is 26.3-only. Caveat: the port ledger notes the full lib suite was still running at close time. |
| 557 | pass | Family: simpleCodec in 26.2; SnowyBlock present in the 26.2 jar (no behaviour method changed). |
| 558 | pass | Family: simpleCodec in 26.2; no behaviour method changed. |
| 559 | pass | Family: simpleCodec in 26.2; no behaviour method changed. |
| 560 | pass | Family: RenderPearl→Blaze3D import renames; body is GPU plumbing, unchanged. |
| 561 | pass | Family: close condition differs (searchBox focus vs isInputCaptured) but the Rust never ports the toggle-key close — unobservable. |
| 562 | pass | ChatVisiblity has the same three values + LEGACY_CODEC in both (javap-verified); 26.3 only adds STREAM_CODEC. |
| 563 | pass | Rust implements the 26.2 deflect forms: reverse scales -0.5 (wind.rs:47-54), aim_deflect sets the unit look (deflect.rs:33-44); MOMENTUM_DEFLECT unported. |
| 564 | pass | Family: EnchantmentPredicate decompiles identically; lambda$static flag is an artifact. |
| 565 | pass | DensityFunction$Axes is a 26.3-only empty marker annotation; nothing in the Rust models Java annotations. |
| 566 | pass | Ported, verified: OIT removed — gfx/oit.rs + shaders deleted, F3+X key removed, improvedTransparency option kept (exists in 26.2); no OitPipelineSet references remain. |
| 567 | pass | Family: extractContent decompiles identically; "changed" flag is an artifact. |
| 568 | pass | Family: Checkbox.extractContents decompiles identically; "changed" flag is an artifact. |
| 569 | pass | Family: simpleCodec in 26.2; no behaviour method changed. |
| 570 | pass | Family: LoggingEventBuilder vs LOGGER.info; emitted line identical; Rust cites the class in a comment only. |
| 571 | pass | Family: rotateDegrees vs mulPose + trailing null arg; same transform, GPU plumbing. |
| 572 | pass | Family: EnchantmentScreen.extractBackground decompiles identically; "changed" flag is an artifact. |
| 573 | pass | Family: player-predicate type change (validateHolder vs validate); Rust ports trigger/test, not validate. |
| 574 | pass | Family: extractIcon decompiles identically; "changed" flag is an artifact. |
| 575 | pass | Family: Blaze3D.openPath vs Util.getPlatform().openPath (same rename as 538). |
| 576 | pass | ItemActivation is 26.3's extraction of 26.2's ScreenEffectRenderer fields; behaviour identical (40 ticks, quintic pose); Rust implements it. |
| 577 | pass | Family: pos.immutable() vs new BlockPos(pos) — the same immutable copy to AllayAi.hearNoteblock. |
| 578 | pass | Ported, verified: armadillo ball-up requires only onGround() (no liquid check in armadillo.rs); 26.3's !isInLiquid() removed. |
| 579 | pass | Family: setBlockAndUpdate vs setBlock(...,3) (item 414). |
| 580 | pass | WoodType POPLAR is 26.3-only (absent in 26.2); Rust has no WoodType enumeration — no surface. |
| 581 | pass | Family: lookupGameProfile decompiles identically; "changed" flag is an artifact. |
| 582 | pass | Ported, verified: METHODS is 85 entries (management_schema.rs:20); four 26.3 world-upgrade notifications removed. |
| 583 | pass | Family: 18 HAS_ABANDONED_CAMP_* tags are 26.3-only; Rust uses only the is_* category tags, same in both. |
| 584 | pass | Family: simpleCodec in 26.2; no behaviour method changed. |
| 585 | pass | SpawnTargetPoint is 26.3-only; Rust implements 26.2's Climate.findSpawnPosition (spawn_finder.rs; 0 0 on empty list). |
| 586 | pass | Family: 26.3 adds a biome pre-check; Rust does not port per-structure findGenerationPoint — unobservable. |
| 587 | pass | Family: same shape as 586; Rust does not port per-structure findGenerationPoint. |
| 588 | pass | Family: biome-query API rename (biomeResolver vs biomeSource); same query, same radius, same check. |
| 589 | pass | Family: RedstoneWireBlock vs RedStoneWireBlock rename only (item 508); trap wiring placements identical. |
| 590 | pass | SteppedInterpolationHandler is 26.3-only; Rust implements 26.2's single InterpolationHandler (interpolation.rs; DEFAULT_INTERPOLATION_STEPS 3). |
| 591 | pass | Family: MenuTabBar.extractWidgetRenderState decompiles identically; "changed" flag is an artifact. Ledger-path note: the named module game/tabs.rs does not exist in the 26.2 tree (MenuTabBar is cited in comments in stats_screen.rs/realms.rs/title.rs) — no behaviour surface. |
| 592 | pass | Family: SystemToast.extractRenderState decompiles identically; "changed" flag is an artifact. Ledger-path note: the module is horizon-client/src/toasts.rs, not game/toasts.rs. |
| 593 | pass | Ported, verified: /test locate is the 26.2 same-level form — 250-block search (test_locate.rs:73), plain /tp, always-numeric distance; cross-dimension search removed. |
| 594 | pass | Family: Prediction.SERVER_ONLY is 26.3-only; Rust models no inventory prediction — already the 26.2 form (menu.rs:924). |
