#!/usr/bin/env python3
"""Compare nether forest vegetation against the 26.2 oracle: the nylium gate, provider draws before the survival check, and scattered placement."""
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
    def vegetation(provider, width=8, height=4):
        return {'type': 'nether_forest_vegetation', 'spread_width': width,
                'spread_height': height, 'state_provider': provider}
    simple = {'type': 'simple_state_provider', 'state': {'Name': 'minecraft:nether_sprouts'}}
    weighted = {'type': 'weighted_state_provider', 'entries': [
        {'data': {'Name': 'minecraft:crimson_roots'}, 'weight': 87},
        {'data': {'Name': 'minecraft:crimson_fungus'}, 'weight': 11},
        {'data': {'Name': 'minecraft:warped_fungus'}, 'weight': 1}]}
    def case(definition=None, **extra):
        origin = extra.pop('origin', [0, 65, 0])
        value = {'definition': json.dumps(wrap(definition if definition is not None else vegetation(simple))),
                 'mode': 'feature', 'origin': origin, 'legacy': False, 'seed': 0, 'world_seed': 1,
                 'fail': -1, 'reject': 0, 'brightness': 15, 'height': origin[1], 'fallback': get('air'),
                 'states': [], 'repeats': 1}
        value.update(extra); return value
    def nylium_floor(name, r=10):
        return [[x, 64, z, get(name)] for x in range(-r, r + 1) for z in range(-r, r + 1)]
    # Nylium floors with both provider kinds, small and real spreads.
    for provider, width, height, legacy, seed in itertools.product(
            [simple, weighted], [3, 8], [1, 4], [False, True], range(4)):
        yield case(vegetation(provider, width, height), legacy=legacy, seed=seed,
                   states=nylium_floor('crimson_nylium'), repeats=2)
    # Warped nylium floor (the tag covers both nylium types).
    for legacy, seed in itertools.product([False, True], range(4)):
        yield case(vegetation(simple), legacy=legacy, seed=seed, states=nylium_floor('warped_nylium'))
    # Gate failure: below the origin is not nylium.
    for legacy, seed in itertools.product([False, True], range(2)):
        yield case(legacy=legacy, seed=seed, states=[[0, 64, 0, get('netherrack')]])
        yield case(legacy=legacy, seed=seed)
    # Rejected writes and injected failures.
    for legacy, fail, reject in itertools.product([False, True], [0, 3], [0, 3]):
        yield case(legacy=legacy, fail=fail, reject=reject, states=nylium_floor('crimson_nylium'))


if __name__ == '__main__':
    harness.run(corpus, 'rust-nether-forest-vegetation-parity', __file__,
        'Native 26.2 NetherForestVegetationFeature: the nylium-tag gate, the build-height gate, provider state draws before the empty/survival checks, and scattered placement over both nylium types. Both RNGs, many seeds, simple and weighted providers, small and real spreads, gate failures, rejected writes and injected failures are covered.')
