#!/usr/bin/env python3
"""Compare basalt columns against the 26.2 oracle: the placement gate, clustered/scattered choice, per-position reach sampling and column building."""
import itertools
import json
try:
    from . import test_simple_block_parity as harness
except ImportError:
    import test_simple_block_parity as harness


def corpus(facts, placement):
    get = lambda name: facts['defaults']['minecraft:' + name]
    small = {'type': 'basalt_columns',
             'height': {'type': 'uniform', 'min_inclusive': 1, 'max_inclusive': 4}, 'reach': 1}
    large = {'type': 'basalt_columns',
             'height': {'type': 'uniform', 'min_inclusive': 5, 'max_inclusive': 10},
             'reach': {'type': 'uniform', 'min_inclusive': 2, 'max_inclusive': 3}}
    def floor(name, y=64, r=12):
        return [[x, y, z, get(name)] for x in range(-r, r + 1) for z in range(-r, r + 1)]
    def case(definition=None, **extra):
        origin = extra.pop('origin', [0, 65, 0])
        value = {'definition': json.dumps(definition if definition is not None else small),
                 'mode': 'feature', 'origin': origin, 'legacy': False, 'seed': 0, 'world_seed': 1,
                 'fail': -1, 'reject': 0, 'brightness': 15, 'height': origin[1], 'fallback': get('air'),
                 'states': [], 'repeats': 1}
        value.update(extra); return value
    # Air above a netherrack floor: columns build upward from the floor.
    for definition, legacy, seed in itertools.product([small, large], [False, True], range(8)):
        yield case(definition, legacy=legacy, seed=seed, states=floor('netherrack'), repeats=2)
    # Lava ocean over a basalt floor: columns build up through the lava.
    for definition, legacy, seed in itertools.product([small, large], [False, True], range(4)):
        yield case(definition, legacy=legacy, seed=seed, sea=66, fallback=get('lava'),
                   states=floor('basalt', y=60) + [[0, 65, 0, get('air')], [0, 64, 0, get('basalt')]])
    # Gate failures: origin on bedrock (cannot place on), origin not air.
    for legacy, seed in itertools.product([False, True], range(2)):
        yield case(legacy=legacy, seed=seed, states=[[0, 65, 0, get('air')], [0, 64, 0, get('bedrock')]])
        yield case(legacy=legacy, seed=seed, fallback=get('netherrack'))
    # Rejected writes and injected failures.
    for legacy, fail, reject in itertools.product([False, True], [0, 3], [0, 3]):
        yield case(legacy=legacy, fail=fail, reject=reject, states=floor('netherrack'))


if __name__ == '__main__':
    harness.run(corpus, 'rust-basalt-columns-parity', __file__,
        'Native 26.2 BasaltColumnsFeature: the air-or-lava-ocean placement gate with its cannot-place-on list, the clustered/scattered draw, per-position reach sampling, randomBetweenClosed cluster positions, find-surface/find-air descent and ascent, and column building through air, lava and existing basalt. Both RNGs, many seeds, lava oceans, gate failures, rejected writes and injected failures are covered.')
