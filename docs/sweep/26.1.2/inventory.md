# Parity sweep — 26.1.2 inventory (the behaviour universe)

Counted 2026-11-25 from 26.1.2's OWN sources only: the vanilla 26.1.2 datagen
reports (`~/workspace/mcir-work/26.1.x/versions/26.1.2/datagen/reports/`),
the 26.1.2 client jar datapack
(`26.1.x/versions/26.1.2/jars/client.jar`, a game jar, sha1-verified against
the research-lock pin `4e618f09…`), and `javap -c` on the NAMED 26.1.2
server-game jar (`26.1.x/versions/26.1.2/jars/server-game.jar`, extracted
from the pinned bundler server jar `97ccd4c0…`, Mojang-mapped names).
DataVersion 4790, protocol 775, Java 25. Machine-readable form:
`inventory-26.1.2.json`, produced by `extract-inventory.py` in this
directory (template from the 26.2 run, parameterised for 26.1, reused
unchanged for 26.1.1 and 26.1.2 — the extractor already falls back to the
old holder classes).

Two 26.1.2-vs-26.2 API facts the extractor had to learn (both verified by
jar listing + javap, both matter for the sweep's oracle triage):

- **26.1.2 predates the `EntityTypes` move, like 26.1 and 26.1.1.**
  Registration still lives in `net.minecraft.world.entity.EntityType` /
  `BlockEntityType` (the 1.21.x shape) — `EntityTypes` / `BlockEntityTypes`
  do not exist in the 26.1.2 jar (verified by jar listing on both jars,
  2026-11-25). The extractor falls back to the old holder classes
  automatically (same putstatic patterns). Oracles written against 26.3's
  `EntityTypes` will fail against the 26.1.2 jar on this alone.
- **26.1.2 predates the shulker/banner collapse and keeps the bed BE
  type.** Dyed shulker boxes are still 17 blocks, banners 32, and
  `minecraft:bed` is a block-entity type covering all 16 bed blocks —
  so 201 blocks carry a block entity (vs 127 in 26.2).

## Headline counts

| Category | Count | Source |
|---|---|---|
| Block types | **1,168** | `registries.json` `minecraft:block` (= `blocks.json` entries) |
| Block states | **29,873** | `blocks.json` states summed |
| Block-entity types | **49** | `minecraft:block_entity_type` (201 distinct blocks carry one) |
| Entity types | **157** | `minecraft:entity_type` |
| Particle types | **117** | `minecraft:particle_type` |
| Fluid types | **5** | empty, flowing_water, water, flowing_lava, lava |
| Item types | **1,506** | `minecraft:item` |
| Biomes | **65** | jar datapack `worldgen/biome` |
| Sound events | **1,902** | `minecraft:sound_event` (sweep checks wiring, not audio) |
| Structures / configured+placed features | 34 / 221+258 | jar datapack `worldgen/structure`, `configured_feature`, `placed_feature` |
| Loot tables / recipes / advancements / tags | 1,326 / 1,515 / 1,617 / 758 | jar datapack file counts |
| Status effects / potions / attributes / enchantments / damage types | 40 / 46 / 35 / 43 / 50 | registries + datapack |

## Entity breakdown (vanilla `MobCategory`, from `EntityType` bytecode)

MONSTER 44, CREATURE 31, WATER_AMBIENT 4, WATER_CREATURE 3,
UNDERGROUND_WATER_CREATURE 1, AXOLOTLS 1, AMBIENT 1, MISC 72. The 72 misc
include boats/rafts, minecarts, projectiles, item/XP, and vanilla's
misc-classified iron_golem, snow_golem, villager and player. **92 of 157
entity types carry attribute sets** (`DefaultAttributes` suppliers).
No `sulfur_cube` (the 26.2 addition) — hence MONSTER 44 vs 26.2's 45.

## Other behaviour-bearing categories (sweep: yes)

Game rules 59, game events 60, menus 25, template pools 188, structure
sets 20, configured carvers 4 (+3 carver types), noises 62, density
functions 35, noise settings 7, world presets 6, flat level presets 9,
data component types 110, command argument types 57, environment
attributes 48, AI brain registries (activities 26 / memory modules 116 /
sensors 24), dimension types 4, chunk statuses 12, trial spawners 28,
trigger types 58, recipe serializers 21, slot displays 11, villager
professions 15, POI types 21, structure NBT files 1,202, processor lists
40. Judged cosmetic/presentation only and OUT of the sweep (same rule as
§1.1): paintings 51, banner patterns 43, trims 18+11, animal variants,
instruments 8, jukebox songs 21, entity models, mob animations, map
decorations, creative tabs, custom stats. PARTIAL: dialogs 3,
advancements 1,617 (triggers only), sound events (wiring only).

## Deltas vs 26.1.1 (the parent version)

None in the behaviour universe: the 26.1.2 inventory is **byte-identical
to 26.1.1's in every count** (verified by diffing `inventory-26.1.2.json`
against `inventory-26.1.1.json` — the only differing key is `version`).
The 26.1.1 → 26.1.2 delta is a DataVersion bump (4788 → 4790, embedded in
the 1,202 structure files) and 6 changed classes (5 methods changed, 1
method and 1 field added); packets, registries, blocks, items and command
nodes are all unchanged (`versions/26.1.2/PORT-NOTES-26.1.2.md`).
The 26.1.2 tree is a delta port of the 26.1.1 tree (34 files differ, none
in the regression sites — see the sweep summary's provenance section), so
the 26.1.1 sweep's defects are re-check-first candidates for 26.1.2 —
verify each fresh against 26.1.2's own jar/data, do not copy 26.1.1's
verdicts. The twelve 26.1 regressions the 26.1.1 sweep found (B1-G05,
B3-G14, B4-G01, B4-G02, B7-G07, B7-G08, B9-G03/G05/G07b/G12/G13) are
expected to reproduce — their code is byte-identical between the two
trees — but each still gets a fresh verdict.

## Inventory caveats

- **Blocks with a block entity: 201, not ~127.** The count is higher
  than 26.2's because 26.1.2 predates the collapse (17 shulker boxes, 32
  banners) and keeps the bed BE type (16 bed blocks). Verified per type
  via `BlockEntityType` bytecode.
- **Items with a behaviour class: not counted.** Needs prepared data
  (`block-behaviours.json`), and no prepared 26.1.2 data exists (see
  Batch 0 setup). Batch 7 resolves it (items.json components as spec,
  or a `BlockItem`-subclass scan of the jar) — same gap as §1.1's 451
  items.
- **No prepared 26.1.2 data exists** (`.local` absent in every worktree).
  The inventory above needs none of it, but Batch 0's driver RUNS and
  Batch 12's data diffs do — Batch 0 covers the setup.
- `registries.json` `minecraft:block_type` exists (as in 1.21.7) with
  unverified provenance — not sized from, same as §1.1.
