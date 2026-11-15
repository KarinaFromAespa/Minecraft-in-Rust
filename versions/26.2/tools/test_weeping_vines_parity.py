#!/usr/bin/env python3
"""Compare weeping vines against the 26.2 oracle: the ceiling gate, roof wart growth and downward hanging vine columns."""
import itertools
import json
try:
    from . import test_simple_block_parity as harness
except ImportError:
    import test_simple_block_parity as harness


def corpus(facts, placement):
    get = lambda name: facts['defaults']['minecraft:' + name]
    vines = {'type': 'weeping_vines'}
    def case(definition=None, **extra):
        origin = extra.pop('origin', [0, 65, 0])
        value = {'definition': json.dumps(definition if definition is not None else vines),
                 'mode': 'feature', 'origin': origin, 'legacy': False, 'seed': 0, 'world_seed': 1,
                 'fail': -1, 'reject': 0, 'brightness': 15, 'height': origin[1], 'fallback': get('air'),
                 'states': [], 'repeats': 1}
        value.update(extra); return value
    def ceiling(name, r=12):
        return [[x, 66, z, get(name)] for x in range(-r, r + 1) for z in range(-r, r + 1)]
    # Each ceiling block type, with a ceiling patch for neighbours.
    for name, legacy, seed in itertools.product(['netherrack', 'nether_wart_block'], [False, True], range(4)):
        yield case(legacy=legacy, seed=seed, states=ceiling(name), repeats=2)
    # Gate failures: origin not empty, ceiling not a valid block.
    for legacy, seed in itertools.product([False, True], range(2)):
        yield case(legacy=legacy, seed=seed, states=[[0, 65, 0, get('stone')]] + ceiling('netherrack'))
        yield case(legacy=legacy, seed=seed, states=ceiling('stone'))
        yield case(legacy=legacy, seed=seed)
    # Rejected writes and injected failures.
    for legacy, fail, reject in itertools.product([False, True], [0, 3], [0, 3]):
        yield case(legacy=legacy, fail=fail, reject=reject, states=ceiling('netherrack'))


if __name__ == '__main__':
    harness.run(corpus, 'rust-weeping-vines-parity', __file__,
        'Native 26.2 WeepingVinesFeature: the empty-origin gate, the netherrack/nether-wart-block ceiling gate, 200 roof wart attempts with the exactly-one-neighbour rule, and 100 hanging vine attempts with height randomisation and downward columns with age 17-25 heads. Both RNGs, many seeds, both ceiling blocks, gate failures, rejected writes and injected failures are covered.')
