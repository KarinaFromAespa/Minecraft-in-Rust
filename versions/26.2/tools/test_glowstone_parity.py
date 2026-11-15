#!/usr/bin/env python3
"""Compare glowstone blobs against the 26.2 oracle: the ceiling gate and 1500 single-neighbour growth attempts."""
import itertools
import json
try:
    from . import test_simple_block_parity as harness
except ImportError:
    import test_simple_block_parity as harness


def wrap(definition):
    # 26.2's ConfiguredFeature codec reads the config from a nested
    # `config` object; the flat form fails with "No key config".
    return {'type': definition['type'],
            'config': {k: v for k, v in definition.items() if k != 'type'}}


def corpus(facts, placement):
    get = lambda name: facts['defaults']['minecraft:' + name]
    glow = {'type': 'glowstone_blob'}
    def case(definition=None, **extra):
        origin = extra.pop('origin', [0, 65, 0])
        value = {'definition': json.dumps(wrap(definition if definition is not None else glow)),
                 'mode': 'feature', 'origin': origin, 'legacy': False, 'seed': 0, 'world_seed': 1,
                 'fail': -1, 'reject': 0, 'brightness': 15, 'height': origin[1], 'fallback': get('air'),
                 'states': [], 'repeats': 1}
        value.update(extra); return value
    # Each ceiling block type grows a blob.
    for ceiling, legacy, seed in itertools.product(['netherrack', 'basalt', 'blackstone'], [False, True], range(4)):
        yield case(legacy=legacy, seed=seed, states=[[0, 66, 0, get(ceiling)]], repeats=2)
    # Gate failures: origin not empty, ceiling not a valid block.
    for legacy, seed in itertools.product([False, True], range(2)):
        yield case(legacy=legacy, seed=seed, states=[[0, 65, 0, get('stone')], [0, 66, 0, get('netherrack')]])
        yield case(legacy=legacy, seed=seed, states=[[0, 66, 0, get('stone')]])
    # Rejected writes and injected failures.
    for legacy, fail, reject in itertools.product([False, True], [0, 3], [0, 3]):
        yield case(legacy=legacy, fail=fail, reject=reject, states=[[0, 66, 0, get('netherrack')]])


if __name__ == '__main__':
    harness.run(corpus, 'rust-glowstone-parity', __file__,
        'Native 26.2 GlowstoneFeature: the empty-origin gate, the netherrack/basalt/blackstone ceiling gate, and 1500 growth attempts that place only in air with exactly one glowstone neighbour. Both RNGs, many seeds, all three ceiling blocks, gate failures, rejected writes and injected failures are covered.')
