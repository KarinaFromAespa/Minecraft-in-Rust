#!/usr/bin/env python3
"""Compare sea pickle placement against the 26.2 oracle: count draws, random offsets, ocean-floor height and 1-4 pickle counts."""
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
    def pickle(count=20): return {'type': 'sea_pickle', 'count': count}
    def case(definition=None, **extra):
        origin = extra.pop('origin', [0, 65, 0])
        value = {'definition': json.dumps(wrap(definition if definition is not None else pickle())),
                 'mode': 'feature', 'origin': origin, 'legacy': False, 'seed': 0, 'world_seed': 1,
                 'fail': -1, 'reject': 0, 'brightness': 15, 'height': origin[1], 'fallback': get('air'),
                 'states': [], 'repeats': 1}
        value.update(extra); return value
    # Water floor with the real 26.2 count (20) and smaller counts.
    for count, legacy, seed in itertools.product([0, 1, 5, 20], [False, True], range(8)):
        yield case(pickle(count), legacy=legacy, seed=seed, fallback=get('water'), repeats=2)
    # Uniform count provider.
    for legacy, seed in itertools.product([False, True], range(8)):
        yield case(pickle({'type': 'uniform', 'min_inclusive': 3, 'max_inclusive': 9}),
                   legacy=legacy, seed=seed, fallback=get('water'))
    # Dry world: no water at any target, immediate false per attempt.
    for legacy, seed in itertools.product([False, True], range(4)):
        yield case(legacy=legacy, seed=seed)
    # Rejected writes and injected failures.
    for legacy, fail, reject in itertools.product([False, True], [0, 3], [0, 3]):
        yield case(legacy=legacy, fail=fail, reject=reject, fallback=get('water'))


if __name__ == '__main__':
    harness.run(corpus, 'rust-sea-pickle-parity', __file__,
        'Native 26.2 SeaPickleFeature: count provider sampling, random +-7 offsets, ocean-floor height, 1-4 pickles per placement and the water + survival gate. Both RNGs, many seeds, constant and uniform counts, rejected writes and injected failures are covered. Fixture height values are controlled and do not update after writes.')
