#!/usr/bin/env python3
"""Compare twisting vines against the 26.2 oracle: the placement gate, first-air-above-ground search, height randomisation and upward columns."""
import itertools
import json
try:
    from . import test_simple_block_parity as harness
except ImportError:
    import test_simple_block_parity as harness


def corpus(facts, placement):
    get = lambda name: facts['defaults']['minecraft:' + name]
    def vines(width=8, height=4, max_height=8):
        return {'type': 'twisting_vines', 'spread_width': width,
                'spread_height': height, 'max_height': max_height}
    def case(definition=None, **extra):
        origin = extra.pop('origin', [0, 65, 0])
        value = {'definition': json.dumps(definition if definition is not None else vines()),
                 'mode': 'feature', 'origin': origin, 'legacy': False, 'seed': 0, 'world_seed': 1,
                 'fail': -1, 'reject': 0, 'brightness': 15, 'height': origin[1], 'fallback': get('air'),
                 'states': [], 'repeats': 1}
        value.update(extra); return value
    def ground(name, r=12):
        return [[x, 64, z, get(name)] for x in range(-r, r + 1) for z in range(-r, r + 1)]
    # Real and bonemeal configs over each valid ground type.
    for definition, name, legacy, seed in itertools.product(
            [vines(), vines(3, 1, 2)], ['netherrack', 'warped_nylium', 'warped_wart_block'],
            [False, True], range(4)):
        yield case(definition, legacy=legacy, seed=seed, states=ground(name), repeats=2)
    # Gate failures: origin not empty, ground below the origin invalid.
    for legacy, seed in itertools.product([False, True], range(2)):
        yield case(legacy=legacy, seed=seed, states=[[0, 65, 0, get('stone')]] + ground('netherrack'))
        yield case(legacy=legacy, seed=seed, states=ground('stone'))
        yield case(legacy=legacy, seed=seed)
    # Rejected writes and injected failures.
    for legacy, fail, reject in itertools.product([False, True], [0, 3], [0, 3]):
        yield case(legacy=legacy, fail=fail, reject=reject, states=ground('netherrack'))


if __name__ == '__main__':
    harness.run(corpus, 'rust-twisting-vines-parity', __file__,
        'Native 26.2 TwistingVinesFeature: the empty-origin/valid-ground gate, the first-air-above-ground descent, height randomisation (double on 1/6, 1 on 1/5) and upward vine columns with age 17-25 heads. Both RNGs, many seeds, real and bonemeal configs, all three ground blocks, gate failures, rejected writes and injected failures are covered.')
