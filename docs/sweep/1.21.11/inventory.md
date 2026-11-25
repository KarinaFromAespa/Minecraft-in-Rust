# Parity sweep — 1.21.11 inventory (the behaviour universe)

Counted 2026-11-25 from 1.21.11's OWN sources only: the vanilla 1.21.11 datagen
reports (`~/workspace/mcir-work/26.1-adapters/versions/1.21.11/datagen/reports/`),
the 1.21.11 client jar datapack
(`e6-deltas/versions/1.21.11/jars/client-named.jar`, Mojang-mapped names,
remapped from the research-lock-pinned client `ba2df812…`), and `javap -c` on
the NAMED 1.21.11 server-game jar
(`e6-deltas/versions/1.21.11/jars/server-game-named.jar`, extracted from the
pinned bundler server jar `64bb6d76…`, Mojang-mapped names).
DataVersion 4671, protocol 774, Java 21, data pack 94.1, resource pack 75.0.
Machine-readable form: `inventory-1.21.11.json`, produced by
`extract-inventory.py` in this directory (template from the 26.2 run,
parameterised for 26.1, reused unchanged for 26.1.1, 26.1.2, 1.21.10 and
1.21.11 — the extractor already falls back to the old holder classes).

Four 1.21.11-vs-26.x API facts the extractor had to learn (all verified by
jar listing + javap, all matter for the sweep's oracle triage):

- **1.21.11 predates the `EntityTypes` move, like 26.1/26.1.1/26.1.2 and
  1.21.10.** Registration still lives in
  `net.minecraft.world.entity.EntityType` / `BlockEntityType` (the 1.21.x
  shape) — `EntityTypes` / `BlockEntityTypes` do not exist in either
  1.21.11 jar (verified by jar listing on both jars, 2026-11-25). The
  extractor falls back to the old holder classes automatically (same
  putstatic patterns). Oracles written against 26.3's `EntityTypes` will
  fail against the 1.21.11 jar on this alone.
- **1.21.11 predates the shulker/banner collapse and keeps the bed BE
  type.** Dyed shulker boxes are still 17 blocks, banners 32, and
  `minecraft:bed` is a block-entity type covering all 16 bed blocks —
  so 201 blocks carry a block entity (vs 127 in 26.2).
- **1.21.11 HAS a `minecraft:environment_attribute` registry (45 entries)
  and a `minecraft:game_rule` registry (59 entries) — both NEW vs
  1.21.10, where both return null.** Environment attributes and the
  gamerule registry arrived in 1.21.11. Game rules are DATA-defined in
  1.21.11 (registry), not code-defined — Batch 9 counts gamerule names
  from the registry and verifies them against `GameRules` bytecode.
  Dimension data in 1.21.11 references environment attributes natively;
  the 1.21.10 "missing environment attributes" pattern inverts here.
- **Slot displays number 8 (26.x: 11)** — the SlotDisplay codec numbering
  gap (26.3 numbering vs 1.21.x numbering) is a known 1.21.11 port caveat
  (see `status.json` notes).

## Headline counts

| Category | Count | Source |
|---|---|---|
| Block types | **1,166** | `registries.json` `minecraft:block` (= `blocks.json` entries) |
| Block states | **29,671** | `blocks.json` states summed |
| Block-entity types | **49** | `minecraft:block_entity_type` (201 distinct blocks carry one) |
| Entity types | **157** | `minecraft:entity_type` |
| Particle types | **115** | `minecraft:particle_type` |
| Fluid types | **5** | empty, flowing_water, water, flowing_lava, lava |
| Item types | **1,505** | `minecraft:item` |
| Biomes | **65** | jar datapack `worldgen/biome` |
| Sound events | **1,838** | `minecraft:sound_event` (sweep checks wiring, not audio) |
| Structures / configured+placed features | 34 / 224+258 | jar datapack `worldgen/structure`, `configured_feature`, `placed_feature` |
| Loot tables / recipes / advancements / tags | 1,324 / 1,470 / 1,584 / 625 | jar datapack file counts |
| Status effects / potions / attributes / enchantments / damage types | 40 / 46 / 35 / 43 / 50 | registries + datapack |

## Entity breakdown (vanilla `MobCategory`, from `EntityType` bytecode)

MONSTER 44, CREATURE 31, WATER_AMBIENT 4, WATER_CREATURE 3,
UNDERGROUND_WATER_CREATURE 1, AXOLOTLS 1, AMBIENT 1, MISC 72. The 72 misc
include boats/rafts, minecarts, projectiles, item/XP, and vanilla's
misc-classified iron_golem, snow_golem, villager and player. **92 of 157
entity types carry attribute sets** (`DefaultAttributes` suppliers).

## Other behaviour-bearing categories (sweep: yes)

Game events 60, menus 25, template pools 188, structure sets 20,
configured carvers 4 (+3 carver types), noises 60, density functions 35,
noise settings 7, world presets 6, flat level presets 9, data component
types 104, command argument types 57, AI brain registries (activities 26 /
memory modules 117 / sensors 24), dimension types 4, chunk statuses 12,
trial spawners 28, trigger types 58, recipe serializers 22, slot displays
8, villager professions 15, POI types 21, structure NBT files 1,202,
processor lists 40, multi-noise biome source parameter lists 2. Judged
cosmetic/presentation only and OUT of the sweep (same rule as §1.1):
paintings 51, banner patterns 43, trims 18+11, animal variants,
instruments 8, jukebox songs 21, entity models, mob animations, map
decorations, creative tabs, custom stats. PARTIAL: dialogs 3,
advancements 1,584 (triggers only), sound events (wiring only).

## Deltas vs 1.21.10 (the template's first child)

Blocks and states are IDENTICAL (1,166 / 29,671); BE types identical
(49, 201 BE blocks); particles identical (115); worldgen identical
(configured features 224, placed 258, noises 60, density functions 35).
The delta is entities +4 (153 → 157: MONSTER 40 → 44, WATER_CREATURE
2 → 3, CREATURE 32 → 31), items +17 (1,488 → 1,505), sound events +67
(1,771 → 1,838), data components +8 (96 → 104), memory modules +7
(110 → 117), sensors −4 (28 → 24), attribute suppliers +4 (88 → 92),
loot +4 (1,320 → 1,324), recipes +9 (1,461 → 1,470), advancements +10
(1,574 → 1,584), tags +13 (612 → 625), enchantments +1 (42 → 43),
damage types +1 (49 → 50), mob effects +1 (39 → 40), trigger types +1
(57 → 58) — PLUS the two registry arrivals: `environment_attribute`
(45 entries, absent in 1.21.10) and `game_rule` (59 entries, absent in
1.21.10). The 1.21.11 → 1.21.10 port is a 719-item behaviour delta
(ranks 10/454/211/42/2) on a nearly identical universe — the sweep's
defects will be behaviour-code deltas, not registry deltas, except
where the two new registries change the port's form.

## Deltas vs 1.21.7 (the same-era predecessor sweep)

1.21.7 counted 1,105 blocks, 27,946 states, 151 entities, 1,416 items,
114 particles. 1.21.11 is bigger on every axis (+61 blocks, +1,725
states, +6 entities, +89 items, +1 particle) — four minor versions of
vanilla content (copper golem/chest/statue, shelf, creaking heart, and
the 1.21.8–1.21.11 additions). The 1.21.7 sweep's defects are the
SECONDARY re-check set for this run — same era, same holder shape, same
vanilla APIs — but each gets a fresh verdict against 1.21.11's own
jar/data; vanilla changed between 1.21.7 and 1.21.11 in places (e.g.
the 1.21.7 beds fix history does not apply here — check the port's bed
BE against 1.21.11's jar directly). The PRIMARY re-check set is the
1.21.10 ledger (379 items) — the template's first child, one protocol
version away.

## Inventory caveats

- **Blocks with a block entity: 201, not ~127.** The count is higher
  than 26.2's because 1.21.11 predates the collapse (17 shulker boxes,
  32 banners) and keeps the bed BE type (16 bed blocks). Verified per
  type via `BlockEntityType` bytecode.
- **Items with a behaviour class: not counted.** Needs prepared data
  (`block-behaviours.json`); Batch 0 establishes whether prepared
  1.21.11 data exists. Batch 7 resolves it (items.json components as
  spec, or a `BlockItem`-subclass scan of the jar).
- **Game rules are DATA-defined in 1.21.11** — the `minecraft:game_rule`
  registry exists with 59 entries. Batch 9 enumerates gamerule names
  from the registry and verifies them against `GameRules` bytecode.
  (In 1.21.10 and earlier 1.21.x, game rules are code-defined; the
  registry arrived in 1.21.11.)
- `registries.json` `minecraft:block_type` exists (as in 1.21.7) with
  unverified provenance — not sized from, same as §1.1.
