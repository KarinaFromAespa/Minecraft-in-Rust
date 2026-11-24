#!/usr/bin/env python3
"""Parity-sweep inventory extraction for one version (template: 26.2,
parameterised for 26.1).

Counts the behaviour universe from the version's OWN sources only:
  - datagen reports (registries.json, blocks.json) for synced-registry counts
  - the version's client jar datapack (data/minecraft/*) for data-driven counts
  - javap -c on the NAMED server-game jar for registration-derived facts
    (entity MobCategory, block-entity valid blocks, attribute suppliers)

26.x note: registration moved out of EntityType/BlockEntityType into
EntityTypes/BlockEntityTypes (static fields + ResourceKey ids), and dyed
shulker boxes / banners collapsed into single blocks with colour components,
so the "blocks with a block entity" count is lower than 1.21.x for the same
BE types. Do NOT compare that count across eras without noting the collapse.

Usage:
  python3 extract-inventory.py --version 26.1 \
      --reports ~/workspace/mcir-work/26.1.x/versions/26.1/datagen/reports \
      --client-jar ~/workspace/mcir-work/26.1.x/versions/26.1/jars/client.jar \
      --server-game-jar ~/workspace/mcir-work/26.1.x/versions/26.1/jars/server-game.jar \
      --javap ~/workspace/tools/jdk-25/bin/javap \
      --out inventory-26.1.json
"""
import argparse
import json
import re
import subprocess
import zipfile
from collections import Counter
from pathlib import Path


def registry_counts(reports):
    reg = json.load(open(reports / "registries.json"))

    def count(key):
        v = reg.get(key)
        if isinstance(v, dict):
            return len(v.get("entries", v))
        return len(v) if isinstance(v, list) else v

    blocks = json.load(open(reports / "blocks.json"))
    states = sum(len(b.get("states", [])) for b in blocks.values())
    out = {
        "block_types": count("minecraft:block"),
        "block_states": states,
        "block_entity_types": count("minecraft:block_entity_type"),
        "entity_types": count("minecraft:entity_type"),
        "particle_types": count("minecraft:particle_type"),
        "fluid_types": count("minecraft:fluid"),
        "item_types": count("minecraft:item"),
        "sound_events": count("minecraft:sound_event"),
        "mob_effects": count("minecraft:mob_effect"),
        "potions": count("minecraft:potion"),
        "attributes": count("minecraft:attribute"),
        "menus": count("minecraft:menu"),
        "data_component_types": count("minecraft:data_component_type"),
        "game_events": count("minecraft:game_event"),
        "villager_professions": count("minecraft:villager_profession"),
        "point_of_interest_types": count("minecraft:point_of_interest_type"),
        "command_argument_types": count("minecraft:command_argument_type"),
        "chunk_statuses": count("minecraft:chunk_status"),
        "game_rules": count("minecraft:game_rule"),
        "environment_attributes": count("minecraft:environment_attribute"),
        "trigger_types": count("minecraft:trigger_type"),
        "recipe_serializers": count("minecraft:recipe_serializer"),
        "recipe_types": count("minecraft:recipe_type"),
        "slot_displays": count("minecraft:slot_display"),
        "activities": count("minecraft:activity"),
        "memory_module_types": count("minecraft:memory_module_type"),
        "sensor_types": count("minecraft:sensor_type"),
        "carvers": count("minecraft:worldgen/carver"),
    }
    be_names = sorted(reg["minecraft:block_entity_type"]["entries"].keys())
    out["block_entity_type_names"] = be_names
    return out


def datapack_counts(client_jar):
    counts = Counter()
    worldgen = Counter()
    with zipfile.ZipFile(client_jar) as z:
        for name in z.namelist():
            if not name.startswith("data/minecraft/") or name.endswith("/"):
                continue
            rest = name[len("data/minecraft/"):]
            top = rest.split("/", 1)[0]
            counts[top] += 1
            if top == "worldgen" and "/" in rest:
                worldgen[rest.split("/")[1]] += 1
    return {"datapack": dict(counts), "worldgen": dict(worldgen)}


def javap(javap_bin, jar, cls):
    proc = subprocess.run([str(javap_bin), "-c", "-p", "-classpath", str(jar), cls],
                          capture_output=True, text=True, timeout=300)
    return proc.stdout


def registration_facts(javap_bin, server_game_jar):
    """Entity categories, BE valid blocks, attribute suppliers — from 26.x
    registration classes (EntityTypes / BlockEntityTypes / DefaultAttributes).
    26.1 PREDATES the move: its registration still lives in EntityType /
    BlockEntityType (the 1.21.x shape), so fall back to those classes when
    the 26.x ones are absent. The putstatic/MobCategory/Blocks patterns are
    identical in both shapes; only the holder class differs. DefaultAttributes
    suppliers reference EntityType.<NAME> in 26.1, EntityTypes.<NAME> in 26.2+.
    """
    txt = javap(javap_bin, server_game_jar, "net.minecraft.world.entity.EntityTypes")
    if "class not found" in txt or "putstatic" not in txt:
        txt = javap(javap_bin, server_game_jar, "net.minecraft.world.entity.EntityType")
    parts = re.split(r"putstatic\s+#\d+\s+// Field (\w+):Lnet/minecraft/world/entity/EntityType;", txt)
    cats = {}
    for i in range(1, len(parts), 2):
        prev = parts[i - 1]
        mcat = re.findall(r"getstatic\s+#\d+\s+// Field net/minecraft/world/entity/MobCategory\.(\w+):", prev)
        if mcat:
            cats[parts[i].lower()] = mcat[-1]

    txt2 = javap(javap_bin, server_game_jar, "net.minecraft.world.level.block.entity.BlockEntityTypes")
    if "class not found" in txt2 or "putstatic" not in txt2:
        txt2 = javap(javap_bin, server_game_jar, "net.minecraft.world.level.block.entity.BlockEntityType")
    parts2 = re.split(r"putstatic\s+#\d+\s+// Field (\w+):Lnet/minecraft/world/level/block/entity/BlockEntityType;", txt2)
    be_blocks = {}
    for i in range(1, len(parts2), 2):
        seg = parts2[i - 1]
        if i == 1:
            j = seg.find("static {};")
            seg = seg[j:] if j >= 0 else seg
        blocks = re.findall(r"getstatic\s+#\d+\s+// Field net/minecraft/world/level/block/Blocks\.(\w+):", seg)
        be_blocks[parts2[i].lower()] = sorted(set(blocks))

    txt3 = javap(javap_bin, server_game_jar, "net.minecraft.world.entity.ai.attributes.DefaultAttributes")
    suppliers = sorted(set(re.findall(
        r"getstatic\s+#\d+\s+// Field net/minecraft/world/entity/EntityTypes?\.(\w+):", txt3)))
    all_be_blocks = set()
    for v in be_blocks.values():
        all_be_blocks.update(v)
    return {
        "entity_category": cats,
        "entity_category_breakdown": dict(Counter(cats.values())),
        "be_blocks": be_blocks,
        "blocks_with_block_entity": len(all_be_blocks),
        "attribute_suppliers": suppliers,
        "entities_with_attribute_suppliers": len(suppliers),
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--version", required=True)
    ap.add_argument("--reports", required=True, type=Path)
    ap.add_argument("--client-jar", required=True, type=Path)
    ap.add_argument("--server-game-jar", required=True, type=Path)
    ap.add_argument("--javap", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    args = ap.parse_args()

    out = {"version": args.version}
    out.update(registry_counts(args.reports))
    out.update(datapack_counts(args.client_jar))
    out.update(registration_facts(args.javap, args.server_game_jar))
    args.out.write_text(json.dumps(out, indent=1))
    print(f"blocks {out['block_types']} states {out['block_states']} "
          f"entities {out['entity_types']} items {out['item_types']} "
          f"particles {out['particle_types']}")
    print(f"entity categories: {len(out['entity_category'])}; "
          f"BE blocks: {out['blocks_with_block_entity']}; "
          f"attribute suppliers: {out['entities_with_attribute_suppliers']}")
    print(f"wrote {args.out}")


if __name__ == "__main__":
    main()
