# Parity sweep — 26.2 inventory (the behaviour universe)

Counted 2026-11-24 from 26.2's OWN sources only: the vanilla 26.2 datagen
reports (`~/workspace/mcir-work/26.2/versions/26.2/datagen/reports/`,
md5-verified distinct from 26.3's dump), the 26.2 client jar datapack
(`leakage-audit/jars/26.2-client.jar`, a game jar), and `javap -c` on the
NAMED 26.2 server-game jar
(`26.1-adapters/versions/26.2/jars/server-game.jar`, 7,753
`net/minecraft` classes, Mojang-mapped names). DataVersion 4903, protocol
776. Machine-readable form: `inventory-26.2.json`, produced by
`extract-inventory.py` in this directory (reusable template for later
versions — parameterise the jar/report paths).

Note on the plan: `docs/PARITY-SWEEP.md` §7 (2026-11-22) says "no named jar
exists on the VM for any 26.x version". That is now outdated: the 26.1/26.2
port work produced named 26.x jars (adapters store, dated Nov 11), and the
26.2 server-game jar is fully Mojang-mapped (classes AND methods — verified:
`net.minecraft.world.level.block.Blocks`, `EntityType.getCategory`, …).
Javap-based checks work directly for 26.2, same method as the 1.21.7 pilot.
The 26.2 server jar in leakage-audit (`26.2-server.jar`, 61 MB) is still a
Mojang BUNDLER jar with no game classes — never javap it; use
`server-game.jar`.

## Headline counts

| Category | Count | Source |
|---|---|---|
| Block types | **1,196** | `registries.json` `minecraft:block` (= `blocks.json` entries) |
| Block states | **32,366** | `blocks.json` states summed |
| Block-entity types | **49** | `minecraft:block_entity_type` (127 distinct blocks carry one) |
| Entity types | **158** | `minecraft:entity_type` |
| Particle types | **125** | `minecraft:particle_type` |
| Fluid types | **5** | empty, flowing_water, water, flowing_lava, lava |
| Item types | **1,537** | `minecraft:item` |
| Biomes | **66** | jar datapack `worldgen/biome` |
| Sound events | **1,968** | `minecraft:sound_event` (sweep checks wiring, not audio) |
| Structures / configured+placed features | 34 / 226+262 | jar datapack `worldgen/structure`, `configured_feature`, `placed_feature` |
| Loot tables / recipes / advancements / tags | 1,355 / 1,585 / 1,688 / 794 | jar datapack file counts |
| Status effects / potions / attributes / enchantments / damage types | 40 / 46 / 40 / 43 / 51 | registries + datapack |

All headline counts agree exactly with `docs/PARITY-SWEEP.md` §1.2 (counted
2026-11-22 from the same sources) — the §1.2 table is confirmed, not just
copied.

## Entity breakdown (vanilla `MobCategory`, from `EntityTypes` bytecode)

MONSTER 45, CREATURE 31, WATER_AMBIENT 4, WATER_CREATURE 3,
UNDERGROUND_WATER_CREATURE 1, AXOLOTLS 1, AMBIENT 1, MISC 72. The 72 misc
include boats/rafts, minecarts, projectiles, item/XP, and vanilla's
misc-classified iron_golem, snow_golem, villager and player. **93 of 158
entity types carry attribute sets** (`DefaultAttributes` suppliers).
`sulfur_cube` (the 26.2 addition) is MONSTER and has an attribute supplier.

26.x registration note: entity types are NOT static fields of `EntityType`
in 26.x — they live in `net.minecraft.world.entity.EntityTypes` (static
fields + `EntityTypeIds` resource keys), and block-entity types in
`BlockEntityTypes`. The extraction script targets that shape; it reports 0s
on a 1.21.x jar, where registration still lives in `EntityType` /
`BlockEntityType`.

## Other behaviour-bearing categories (sweep: yes)

Game rules 59, game events 61, menus 25, template pools 188, structure
sets 20, configured carvers 4 (+3 carver types), noises 63, density
functions 35, noise settings 7, world presets 7, flat level presets 9,
data component types 111, command argument types 57, environment
attributes 48, AI brain registries (activities 26 / memory modules 116 /
sensors 24), dimension types 4, chunk statuses 12, trial spawners 28,
trigger types 58, recipe serializers 21, slot displays 11, villager
professions 15, POI types 21, structure NBT files 1,212, processor lists
40, sulfur-cube archetypes 12. Judged cosmetic/presentation only and OUT
of the sweep (same rule as §1.1): paintings 51, banner patterns 43, trims
18+11, animal variants, instruments 8, jukebox songs 22, entity models,
mob animations, map decorations, creative tabs, custom stats. PARTIAL:
dialogs 3, advancements 1,688 (triggers only), sound events (wiring only).

## Deltas vs 26.1 (the parent version)

26.2 adds the cinnabar/sulfur block families (+28 blocks), the
`sulfur_cube` entity, 8 geyser/sulfur particles (117→125), 5 attributes
(35→40) and a 66th biome. Its BE types swap `bed` OUT for `potent_sulfur`
(net still 49) — `minecraft:bed` is NOT a 26.2 block-entity type (verified
in the registry and in `BlockEntityTypes` bytecode; `potent_sulfur` covers
only `POTENT_SULFUR`). The 26.1 port's finding that 26.1 HAS a bed BE type
is a genuine 26.1↔26.2 difference, not a sweep artifact.

## Inventory caveats

- **Blocks with a block entity: 127, not ~201.** The drop from 1.21.x is
  real, not an extraction error: 26.x collapsed dyed shulker boxes (17
  blocks → `SHULKER_BOX` + `DYED_SHULKER_BOX`) and banners (32 → `BANNER`
  + `WALL_BANNER`) into single blocks with colour components, and removed
  the bed BE type (16 bed blocks). Verified per type (sign 24, hanging
  sign 24, skull 14, shelf 12, chest 2 incl. copper chest).
- **Items with a behaviour class: not counted.** Needs prepared data
  (`block-behaviours.json`), and no prepared 26.2 data exists (see Batch 0
  setup). Batch 7 resolves it (items.json components as spec, or a
  `BlockItem`-subclass scan of the jar) — same gap as §1.1's 451 items.
- **No prepared 26.2 data exists** (`.local` deleted; §7 verdict NEEDS
  PREPARE). The inventory above needs none of it, but Batch 0's driver
  RUNS and Batch 12's data diffs do — Batch 0 covers the setup.
- `registries.json` `minecraft:block_type` exists (as in 1.21.7) with
  unverified provenance — not sized from, same as §1.1.
