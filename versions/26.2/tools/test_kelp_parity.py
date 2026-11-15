#!/usr/bin/env python3
"""Compare kelp columns against the 26.2 oracle: ocean-floor gate, water checks, height draws, age draws and early-termination re-cap."""
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
    kelp = {'type': 'kelp'}
    def case(definition=None, **extra):
        origin = extra.pop('origin', [0, 65, 0])
        value = {'definition': json.dumps(wrap(definition if definition is not None else kelp)),
                 'mode': 'feature', 'origin': origin, 'legacy': False, 'seed': 0, 'world_seed': 1,
                 'fail': -1, 'reject': 0, 'brightness': 15, 'height': origin[1], 'fallback': get('air'),
                 'states': [], 'repeats': 1}
        value.update(extra); return value
    water_column = [[0, y, 0, get('water')] for y in range(60, 80)]
    # Full water column: the stalk grows to its drawn height and caps.
    for legacy, seed in itertools.product([False, True], range(16)):
        yield case(legacy=legacy, seed=seed, fallback=get('water'), states=water_column, repeats=2)
    # Stone obstacle mid-column: early termination re-caps the block below.
    for legacy, seed, stop in itertools.product([False, True], range(8), [66, 68, 72]):
        states = [[0, y, 0, get('water')] for y in range(60, 80)]
        states.append([0, stop, 0, get('stone')])
        yield case(legacy=legacy, seed=seed, fallback=get('water'), states=states)
    # Water at the floor height but air above: the i=0 check fails, the
    # i=1 else-branch re-caps below.
    for legacy, seed in itertools.product([False, True], range(8)):
        yield case(legacy=legacy, seed=seed, states=[[0, 65, 0, get('water')]])
    # No water at the floor height: immediate false, no draws.
    for legacy, seed in itertools.product([False, True], range(4)):
        yield case(legacy=legacy, seed=seed)
    # Floor height differs from the origin height.
    for legacy, seed in itertools.product([False, True], range(4)):
        yield case(legacy=legacy, seed=seed, origin=[0, 70, 0], height=65,
                   fallback=get('water'), states=water_column)
    # Rejected writes and injected failures.
    for legacy, fail, reject in itertools.product([False, True], [0, 3, 25], [0, 3]):
        yield case(legacy=legacy, fail=fail, reject=reject, fallback=get('water'), states=water_column)


if __name__ == '__main__':
    harness.run(corpus, 'rust-kelp-parity', __file__,
        'Native 26.2 KelpFeature: ocean-floor height gate, water checks at the stalk and above it, 1 + nextInt(10) height, age 20-23 head, and the early-termination re-cap of the block below when the column is cut short. Both RNGs, many seeds, obstacle heights, rejected writes and injected failures are covered. Fixture height values are controlled and do not update after writes.')
