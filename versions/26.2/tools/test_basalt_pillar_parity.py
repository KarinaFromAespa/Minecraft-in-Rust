#!/usr/bin/env python3
"""Compare basalt pillars against the 26.2 oracle: ceiling gate, descent with directional hang-offs, base hang-offs and the spread base."""
import itertools
import json
try:
    from . import test_simple_block_parity as harness
except ImportError:
    import test_simple_block_parity as harness


def corpus(facts, placement):
    get = lambda name: facts['defaults']['minecraft:' + name]
    pillar = {'type': 'basalt_pillar'}
    def case(definition=None, **extra):
        origin = extra.pop('origin', [0, 65, 0])
        value = {'definition': json.dumps(definition if definition is not None else pillar),
                 'mode': 'feature', 'origin': origin, 'legacy': False, 'seed': 0, 'world_seed': 1,
                 'fail': -1, 'reject': 0, 'brightness': 15, 'height': origin[1], 'fallback': get('air'),
                 'states': [], 'repeats': 1}
        value.update(extra); return value
    # Ceiling with a floor 6 below: full pillar, hang-offs and base spread.
    for legacy, seed in itertools.product([False, True], range(8)):
        states = [[0, 66, 0, get('netherrack')]]
        states += [[x, 59, z, get('netherrack')] for x in range(-5, 6) for z in range(-5, 6)]
        yield case(legacy=legacy, seed=seed, states=states, repeats=2)
    # No floor: the pillar descends out of build height and returns early.
    for legacy, seed in itertools.product([False, True], range(2)):
        yield case(legacy=legacy, seed=seed, states=[[0, 66, 0, get('netherrack')]])
    # Gate failures: origin not empty, nothing above the origin.
    for legacy, seed in itertools.product([False, True], range(2)):
        yield case(legacy=legacy, seed=seed, states=[[0, 65, 0, get('stone')], [0, 66, 0, get('netherrack')]])
        yield case(legacy=legacy, seed=seed)
    # Rejected writes and injected failures.
    for legacy, fail, reject in itertools.product([False, True], [0, 3], [0, 3]):
        states = [[0, 66, 0, get('netherrack')]]
        states += [[x, 59, z, get('netherrack')] for x in range(-5, 6) for z in range(-5, 6)]
        yield case(legacy=legacy, fail=fail, reject=reject, states=states)


if __name__ == '__main__':
    harness.run(corpus, 'rust-basalt-pillar-parity', __file__,
        'Native 26.2 BasaltPillarFeature: the empty-origin/solid-ceiling gate, descent with short-circuiting directional hang-offs, base hang-offs, the 7x7 spread base with its distance-weighted chance and 3-block drop, and the out-of-build-height early return. Both RNGs, many seeds, gate failures, rejected writes and injected failures are covered.')
