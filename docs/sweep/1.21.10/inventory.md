# Parity sweep — 1.21.10 inventory (the behaviour universe)

Counted 2026-11-25 from 1.21.10's OWN sources only: the vanilla 1.21.10 datagen
reports (`~/workspace/mcir-work/26.1-adapters/versions/1.21.10/datagen/reports/`),
the 1.21.10 client jar datapack
(`e6-deltas/versions/1.21.10/jars/client-named.jar`, Mojang-mapped names,
remapped from the research-lock-pinned client `d3bdf582…`), and `javap -c` on
the NAMED 1.21.10 server-game jar
(`e6-deltas/versions/1.21.10/jars/server-game-named.jar`, extracted from the
pinned bundler server jar `95495a7f…`, Mojang-mapped names).
DataVersion 4556, protocol 773, Java 21, data pack 88.0, resource pack 69.0.
Machine-readable form: `inventory-1.21.10.json`, produced by
`extract-inventory.py` in this directory (template from the 26.2 run,
parameterised for 26.1, reused unchanged for 26.1.1, 26.1.2 and 1.21.10 —
the extractor already falls back to the old holder classes).

Three 1.21.10-vs-26.x API facts the extractor had to learn (all verified by
jar listing + javap, all matter for the sweep's oracle triage):

- **1.21.10 predates the `EntityTypes` move, like 26.1/26.1.1/26.1.2.**
  Registration still lives in `net.minecraft.world.entity.EntityType` /
  `BlockEntityType` (the 1.21.x shape) — `EntityTypes` / `BlockEntityTypes`
  do not exist in either 1.21.10 jar (verified by jar listing on both jars,
  2026-11-25). The extractor falls back to the old holder classes
  automatically (same putstatic patterns). Oracles written against 26.3's
  `EntityTypes` will fail against the 1.21.10 jar on this alone.
- **1.21.10 predates the shulker/banner collapse and keeps the bed BE
  type.** Dyed shulker boxes are still 17 blocks, banners 32, and
  `minecraft:bed` is a block-entity type covering all 16 bed blocks —
  so 201 blocks carry a block entity (vs 127 in 26.2).
- **1.21.10 has NO `minecraft:environment_attribute` registry and NO
  `minecraft:game_rule` registry** (both return null from
  `registries.json`). Environment attributes are a 26.x addition; game
  rules are code-defined in 1.21.x (`GameRules` bytecode, not data).
  Any port code or oracle that reads either registry is 26.x-form and
  wrong for this version. Slot displays number 8 (26.x: 11) — the
  SlotDisplay codec numbering gap (26.3 numbering vs 1.21.x numbering)
  is a known 1.21.10 port caveat (see `status.json` notes).

## Headline counts

| Category | Count | Source |
|---|---|---|
| Block types | **1,166** | `registries.json` `minecraft:block` (= `blocks.json` entries) |
| Block states | **29,671** | `blocks.json` states summed |
| Block-entity types | **49** | `minecraft:block_entity_type` (201 distinct blocks carry one) |
| Entity types | **153** | `minecraft:entity_type` |
| Particle types | **115** | `minecraft:particle_type` |
| Fluid types | **5** | empty, flowing_water, water, flowing_lava, lava |
| Item types | **1,488** | `minecraft:item` |
| Biomes | **65** | jar datapack `worldgen/biome` |
| Sound events | **1,771** | `minecraft:sound_event` (sweep checks wiring, not audio) |
| Structures / configured+placed features | 34 / 224+258 | jar datapack `worldgen/structure`, `configured_feature`, `placed_feature` |
| Loot tables / recipes / advancements / tags | 1,320 / 1,461 / 1,574 / 612 | jar datapack file counts |
| Status effects / potions / attributes / enchantments / damage types | 39 / 46 / 35 / 42 / 49 | registries + datapack |

## Entity breakdown (vanilla `MobCategory`, from `EntityType` bytecode)

MONSTER 40, CREATURE 32, WATER_AMBIENT 4, WATER_CREATURE 2,
UNDERGROUND_WATER_CREATURE 1, AXOLOTLS 1, AMBIENT 1, MISC 72. The 72 misc
include boats/rafts, minecarts, projectiles, item/XP, and vanilla's
misc-classified iron_golem, snow_golem, villager and player. **88 of 153
entity types carry attribute sets** (`DefaultAttributes` suppliers).

## Other behaviour-bearing categories (sweep: yes)

Game events 60, menus 25, template pools 188, structure sets 20,
configured carvers 4 (+3 carver types), noises 60, density functions 35,
noise settings 7, world presets 6, flat level presets 9, data component
types 96, command argument types 57, AI brain registries (activities 26 /
memory modules 110 / sensors 28), dimension types 4, chunk statuses 12,
trial spawners 28, trigger types 57, recipe serializers 22, slot displays
8, villager professions 15, POI types 21, structure NBT files 1,202,
processor lists 40, multi-noise biome source parameter lists 2. Judged
cosmetic/presentation only and OUT of the sweep (same rule as §1.1):
paintings 51, banner patterns 43, trims 18+11, animal variants,
instruments 8, jukebox songs 21, entity models, mob animations, map
decorations, creative tabs, custom stats. PARTIAL: dialogs 3,
advancements 1,574 (triggers only), sound events (wiring only).

## Deltas vs 1.21.11 (the era template)

Blocks and states are IDENTICAL (1,166 / 29,671); BE types identical
(49, 201 BE blocks); particles identical (115); worldgen identical
(configured features 224, placed 258, noises 60, density functions 35).
The delta is entities −4 (157 → 153: MONSTER 44 → 40, WATER_CREATURE
3 → 2, CREATURE 31 → 32), items −17 (1,505 → 1,488), sound events −67
(1,838 → 1,771), data components −8 (104 → 96), memory modules −7
(117 → 110), sensors +4 (24 → 28), attribute suppliers −4 (92 → 88),
loot −4 (1,324 → 1,320), recipes −9 (1,470 → 1,461), advancements −10
(1,584 → 1,574). The 1.21.11 → 1.21.10 port is a 719-item behaviour
delta (ranks 10/454/211/42/2) on a nearly identical universe — the
sweep's defects will be behaviour-code deltas, not registry deltas.

## Deltas vs 1.21.7 (the same-era predecessor sweep)

1.21.7 counted 1,105 blocks, 27,946 states, 151 entities, 1,416 items,
114 particles. 1.21.10 is bigger on every axis (+61 blocks, +1,725
states, +2 entities, +72 items, +1 particle) — three minor versions of
vanilla content (copper golem/chest/statue, shelf, creaking heart, and
the 1.21.8–1.21.10 additions). The 1.21.7 sweep's defects are the
PRIMARY re-check set for this run — same era, same holder shape, same
vanilla APIs — but each gets a fresh verdict against 1.21.10's own
jar/data; vanilla changed between 1.21.7 and 1.21.10 in places (e.g.
the 1.21.7 beds fix history does not apply here — check the port's bed
BE against 1.21.10's jar directly).

## Inventory caveats

- **Blocks with a block entity: 201, not ~127.** The count is higher
  than 26.2's because 1.21.10 predates the collapse (17 shulker boxes,
  32 banners) and keeps the bed BE type (16 bed blocks). Verified per
  type via `BlockEntityType` bytecode.
- **Items with a behaviour class: not counted.** Needs prepared data
  (`block-behaviours.json`); Batch 0 establishes whether prepared
  1.21.10 data exists. Batch 7 resolves it (items.json components as
  spec, or a `BlockItem`-subclass scan of the jar).
- **Game rules are code-defined** — no registry count exists. Batch 9
  enumerates gamerule names from `GameRules` bytecode.
- `registries.json` `minecraft:block_type` exists (as in 1.21.7) with
  unverified provenance — not sized from, same as §1.1.
