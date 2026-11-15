#!/usr/bin/env python3
"""Compare seagrass placement against the 26.2 oracle: random offsets, ocean-floor height, tall/short choice and the tall-plant water gate."""
import itertools
import json
try:
    from . import test_simple_block_parity as harness
except ImportError:
    import test_simple_block_parity as harness


def corpus(facts, placement):
    get = lambda name: facts['defaults']['minecraft:' + name]
    def seagrass(probability=0.6): return {'type': 'seagrass', 'probability': probability}
    def case(definition=None, **extra):
        origin = extra.pop('origin', [0, 65, 0])
        value = {'definition': json.dumps(definition if definition is not None else seagrass()),
                 'mode': 'feature', 'origin': origin, 'legacy': False, 'seed': 0, 'world_seed': 1,
                 'fail': -1, 'reject': 0, 'brightness': 15, 'height': origin[1], 'fallback': get('air'),
                 'states': [], 'repeats': 1}
        value.update(extra); return value
    # Water floor: short and tall plants across probabilities and seeds.
    for probability, legacy, seed in itertools.product([0, 0.2, 0.6, 1], [False, True], range(16)):
        yield case(seagrass(probability), legacy=legacy, seed=seed, fallback=get('water'), repeats=2)
    # Dry world: no water at the target, immediate false.
    for legacy, seed in itertools.product([False, True], range(4)):
        yield case(legacy=legacy, seed=seed)
    # Tall plant with a dry block above the target: nothing is written but
    # vanilla still reports success.
    for legacy, seed in itertools.product([False, True], range(8)):
        yield case(seagrass(1), legacy=legacy, seed=seed, fallback=get('water'),
                   states=[[0, 66, 0, get('air')]])
    # Invalid probabilities: decode errors on both sides.
    for value in [None, -1, -0.0, 1.0000000001, 1.01, '0.5', [], {}]:
        yield case(seagrass(value))
    yield case({'type': 'seagrass'})
    # Rejected writes and injected failures.
    for legacy, fail, reject in itertools.product([False, True], [0, 3], [0, 3]):
        yield case(legacy=legacy, fail=fail, reject=reject, fallback=get('water'))


if __name__ == '__main__':
    harness.run(corpus, 'rust-seagrass-parity', __file__,
        'Native 26.2 SeagrassFeature: random +-7 offsets, ocean-floor height, the tall/short probability draw, the tall upper-block water gate and the placedAny quirk (a tall plant whose upper block is dry still reports success). Both RNGs, many seeds, invalid probabilities, rejected writes and injected failures are covered. Fixture height values are controlled and do not update after writes.')
