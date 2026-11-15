#!/usr/bin/env python3
"""Compare desert wells against the 26.2 oracle: the sand landing, the hollow check, the full structure and the seeded suspicious sand."""
import itertools
import json
try:
    from . import test_simple_block_parity as harness
except ImportError:
    import test_simple_block_parity as harness


def corpus(facts, placement):
    get = lambda name: facts['defaults']['minecraft:' + name]
    well = {'type': 'desert_well'}
    def case(definition=None, **extra):
        origin = extra.pop('origin', [0, 70, 0])
        value = {'definition': json.dumps(definition if definition is not None else well),
                 'mode': 'feature', 'origin': origin, 'legacy': False, 'seed': 0, 'world_seed': 1,
                 'fail': -1, 'reject': 0, 'brightness': 15, 'height': origin[1], 'fallback': get('air'),
                 'states': [], 'repeats': 1}
        value.update(extra); return value
    def ground():
        states = []
        for x in range(-4, 5):
            for z in range(-4, 5):
                states.append([x, 64, z, get('sand')])
                states.append([x, 63, z, get('sandstone')])
                states.append([x, 62, z, get('sandstone')])
        return states
    # Full wells with and without a loot container; seeds vary the sus-sand picks.
    for loot, legacy, seed in itertools.product([None, 'chest'], [False, True], range(8)):
        extra = {'states': ground()}
        if loot is not None:
            extra['loot_entity'] = loot
        yield case(legacy=legacy, seed=seed, repeats=2, **extra)
    # Gate failures: landing on stone, a hollow column under the well.
    for legacy, seed in itertools.product([False, True], range(2)):
        states = ground()
        states.append([0, 64, 0, get('stone')])
        yield case(legacy=legacy, seed=seed, states=states)
        states = [s for s in ground() if not (s[0] == 1 and s[2] == 1 and s[1] in (62, 63))]
        yield case(legacy=legacy, seed=seed, states=states)
    # Rejected writes and injected failures.
    for legacy, fail, reject in itertools.product([False, True], [0, 3], [0, 3]):
        yield case(legacy=legacy, fail=fail, reject=reject, states=ground(), loot_entity='chest')


if __name__ == '__main__':
    harness.run(corpus, 'rust-desert-well-parity', __file__,
        'Native 26.2 DesertWellFeature: descent to the sand landing, the hollow-column check, the 5x5x3 sandstone base, water and sand crosses, slabbed rim and roof, four pillars, and two suspicious-sand blocks seeded with the desert-well archaeology loot table at pos.asLong(). Both RNGs, many seeds, loot and no-loot worlds, gate failures, rejected writes and injected failures are covered.')
