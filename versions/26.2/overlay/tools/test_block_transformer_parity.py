#!/usr/bin/env python3
"""Compare the hardcoded axe/hoe/shovel block transformers with vanilla.

The Java oracle (VanillaBlockTransformerOracle) evaluates AxeItem, HoeItem,
ShovelItem and HoneycombItem's tables; the Rust example
(block_transform_fixture) evaluates horizon_client::block_transform. Cases are
every block state any table mentions (plus a random sample of all states)
under each transformer, with a few blocks above and three clicked faces.
"""
import argparse
import random
try:
    from . import interaction_lab as lib, lab
except ImportError:
    import interaction_lab as lib
    import lab

ABOVE = ['minecraft:air|', 'minecraft:cave_air|', 'minecraft:stone|', 'minecraft:water|level=0', 'minecraft:oak_slab|type=bottom,waterlogged=false']
FACES = ['up', 'down', 'north']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--seed', type=int, default=7)
    parser.add_argument('--sample', type=int, default=3000)
    parser.add_argument('--limit', type=int, default=4)
    args = parser.parse_args()
    work = lib.work_dir('block-transformer-parity')
    vanilla = lib.Vanilla(work)
    vanilla.compile('ParityData', 'VanillaBlockTransformerOracle')
    vanilla.run('VanillaBlockTransformerOracle', 'dump', work / 'states.txt')
    states = (work / 'states.txt').read_text().splitlines()
    # 26.2 has no block_transformer data and no turns_into_* tags: the
    # cases come from the hardcoded tables (strip/till/flatten blocks, the
    # copper families, campfires) plus a random sample of all states.
    mentioned = {
        'minecraft:oak_wood', 'minecraft:oak_log', 'minecraft:dark_oak_wood',
        'minecraft:dark_oak_log', 'minecraft:pale_oak_wood', 'minecraft:pale_oak_log',
        'minecraft:acacia_wood', 'minecraft:acacia_log', 'minecraft:cherry_wood',
        'minecraft:cherry_log', 'minecraft:birch_wood', 'minecraft:birch_log',
        'minecraft:jungle_wood', 'minecraft:jungle_log', 'minecraft:spruce_wood',
        'minecraft:spruce_log', 'minecraft:warped_stem', 'minecraft:warped_hyphae',
        'minecraft:crimson_stem', 'minecraft:crimson_hyphae', 'minecraft:mangrove_wood',
        'minecraft:mangrove_log', 'minecraft:bamboo_block', 'minecraft:grass_block',
        'minecraft:dirt_path', 'minecraft:dirt', 'minecraft:coarse_dirt',
        'minecraft:rooted_dirt', 'minecraft:podzol', 'minecraft:mycelium',
        'minecraft:campfire', 'minecraft:soul_campfire',
    }
    rng = random.Random(args.seed)
    chosen = [s for s in states if s.split('|')[0] in mentioned or 'copper' in s or 'lightning_rod' in s]
    chosen += rng.sample(states, args.sample)
    cases = []
    for transformer in ['axe', 'hoe', 'shovel', 'honeycomb']:
        for state in chosen:
            for face in FACES:
                cases.append('\t'.join([transformer, face, state, rng.choice(ABOVE)]))
    (work / 'cases.txt').write_text(''.join(c + '\n' for c in cases))
    vanilla.run('VanillaBlockTransformerOracle', work / 'cases.txt', work / 'native.txt')
    lib.cargo_example('horizon-client', 'block_transform_fixture', lib.RESOURCES, work / 'cases.txt', work / 'rust.txt')
    total = lib.compare(cases, work / 'native.txt', work / 'rust.txt',
                        lambda case, line: case.split('\t')[0] + ' ' + line.split('\t')[0][:12], None, args.limit)
    raise SystemExit(1 if total else 0)


if __name__ == '__main__':
    main()
