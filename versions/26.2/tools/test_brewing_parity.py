#!/usr/bin/env python3
"""Compare the brewing mixes' slot rules, matching and assembled output with vanilla.

26.2 has no `minecraft:brewing` recipe files: brewing is `PotionBrewing`'s
hardcoded mixes (`addVanillaMixes`). The Java oracle builds a `PotionBrewing`
over the vanilla registries, checks the three brewing-slot filters
(`isIngredient`, the bottle items, the `brewing_fuel` tag) and calls `mix`.
The Rust probe calls `brewing::Brewing::is_bottle`/`is_reagent`,
`brewing_fuel` and `Brewing::find`. Cases cover every vanilla mix, every
potion type and potion id against every reagent, candidates from the whole
item registry, and non-matching pairs. Tick cases run
`BrewingStandBlockEntity.serverTick` and `BrewingStand::tick` for 0..8002
ticks: the flat 400-tick brew, blaze powder's 20 fuel uses, fuel loaded with
nothing to brew, splash and lingering outputs, ingredient shrink and
mismatch, the brew sound, and the bottle bits. `--no-tick` leaves the tick
corpus out, and `--baseline` keeps only the exact mix pairs.
"""
import json
from pathlib import Path
import sys
try:
    from . import lab, oracle_harness as harness
except ImportError:
    import lab
    import oracle_harness as harness

POTION_ITEMS = ['potion', 'splash_potion', 'lingering_potion']

# `PotionBrewing.addVanillaMixes` (26.2): (from potion, reagent, to potion),
# in registration order. `addStartMix` contributes its water->mundane pair
# first, then the awkward->potion pair.
POTION_MIXES = [
    ('minecraft:water', 'minecraft:glowstone_dust', 'minecraft:thick'),
    ('minecraft:water', 'minecraft:redstone', 'minecraft:mundane'),
    ('minecraft:water', 'minecraft:nether_wart', 'minecraft:awkward'),
    ('minecraft:water', 'minecraft:breeze_rod', 'minecraft:mundane'),
    ('minecraft:awkward', 'minecraft:breeze_rod', 'minecraft:wind_charged'),
    ('minecraft:water', 'minecraft:slime_block', 'minecraft:mundane'),
    ('minecraft:awkward', 'minecraft:slime_block', 'minecraft:oozing'),
    ('minecraft:water', 'minecraft:stone', 'minecraft:mundane'),
    ('minecraft:awkward', 'minecraft:stone', 'minecraft:infested'),
    ('minecraft:water', 'minecraft:cobweb', 'minecraft:mundane'),
    ('minecraft:awkward', 'minecraft:cobweb', 'minecraft:weaving'),
    ('minecraft:awkward', 'minecraft:golden_carrot', 'minecraft:night_vision'),
    ('minecraft:night_vision', 'minecraft:redstone', 'minecraft:long_night_vision'),
    ('minecraft:night_vision', 'minecraft:fermented_spider_eye', 'minecraft:invisibility'),
    ('minecraft:long_night_vision', 'minecraft:fermented_spider_eye', 'minecraft:long_invisibility'),
    ('minecraft:invisibility', 'minecraft:redstone', 'minecraft:long_invisibility'),
    ('minecraft:water', 'minecraft:magma_cream', 'minecraft:mundane'),
    ('minecraft:awkward', 'minecraft:magma_cream', 'minecraft:fire_resistance'),
    ('minecraft:fire_resistance', 'minecraft:redstone', 'minecraft:long_fire_resistance'),
    ('minecraft:water', 'minecraft:rabbit_foot', 'minecraft:mundane'),
    ('minecraft:awkward', 'minecraft:rabbit_foot', 'minecraft:leaping'),
    ('minecraft:leaping', 'minecraft:redstone', 'minecraft:long_leaping'),
    ('minecraft:leaping', 'minecraft:glowstone_dust', 'minecraft:strong_leaping'),
    ('minecraft:leaping', 'minecraft:fermented_spider_eye', 'minecraft:slowness'),
    ('minecraft:long_leaping', 'minecraft:fermented_spider_eye', 'minecraft:long_slowness'),
    ('minecraft:slowness', 'minecraft:redstone', 'minecraft:long_slowness'),
    ('minecraft:slowness', 'minecraft:glowstone_dust', 'minecraft:strong_slowness'),
    ('minecraft:awkward', 'minecraft:turtle_helmet', 'minecraft:turtle_master'),
    ('minecraft:turtle_master', 'minecraft:redstone', 'minecraft:long_turtle_master'),
    ('minecraft:turtle_master', 'minecraft:glowstone_dust', 'minecraft:strong_turtle_master'),
    ('minecraft:swiftness', 'minecraft:fermented_spider_eye', 'minecraft:slowness'),
    ('minecraft:long_swiftness', 'minecraft:fermented_spider_eye', 'minecraft:long_slowness'),
    ('minecraft:water', 'minecraft:sugar', 'minecraft:mundane'),
    ('minecraft:awkward', 'minecraft:sugar', 'minecraft:swiftness'),
    ('minecraft:swiftness', 'minecraft:redstone', 'minecraft:long_swiftness'),
    ('minecraft:swiftness', 'minecraft:glowstone_dust', 'minecraft:strong_swiftness'),
    ('minecraft:awkward', 'minecraft:pufferfish', 'minecraft:water_breathing'),
    ('minecraft:water_breathing', 'minecraft:redstone', 'minecraft:long_water_breathing'),
    ('minecraft:water', 'minecraft:glistering_melon_slice', 'minecraft:mundane'),
    ('minecraft:awkward', 'minecraft:glistering_melon_slice', 'minecraft:healing'),
    ('minecraft:healing', 'minecraft:glowstone_dust', 'minecraft:strong_healing'),
    ('minecraft:healing', 'minecraft:fermented_spider_eye', 'minecraft:harming'),
    ('minecraft:strong_healing', 'minecraft:fermented_spider_eye', 'minecraft:strong_harming'),
    ('minecraft:harming', 'minecraft:glowstone_dust', 'minecraft:strong_harming'),
    ('minecraft:poison', 'minecraft:fermented_spider_eye', 'minecraft:harming'),
    ('minecraft:long_poison', 'minecraft:fermented_spider_eye', 'minecraft:harming'),
    ('minecraft:strong_poison', 'minecraft:fermented_spider_eye', 'minecraft:strong_harming'),
    ('minecraft:water', 'minecraft:spider_eye', 'minecraft:mundane'),
    ('minecraft:awkward', 'minecraft:spider_eye', 'minecraft:poison'),
    ('minecraft:poison', 'minecraft:redstone', 'minecraft:long_poison'),
    ('minecraft:poison', 'minecraft:glowstone_dust', 'minecraft:strong_poison'),
    ('minecraft:water', 'minecraft:ghast_tear', 'minecraft:mundane'),
    ('minecraft:awkward', 'minecraft:ghast_tear', 'minecraft:regeneration'),
    ('minecraft:regeneration', 'minecraft:redstone', 'minecraft:long_regeneration'),
    ('minecraft:regeneration', 'minecraft:glowstone_dust', 'minecraft:strong_regeneration'),
    ('minecraft:water', 'minecraft:blaze_powder', 'minecraft:mundane'),
    ('minecraft:awkward', 'minecraft:blaze_powder', 'minecraft:strength'),
    ('minecraft:strength', 'minecraft:redstone', 'minecraft:long_strength'),
    ('minecraft:strength', 'minecraft:glowstone_dust', 'minecraft:strong_strength'),
    ('minecraft:water', 'minecraft:fermented_spider_eye', 'minecraft:weakness'),
    ('minecraft:weakness', 'minecraft:redstone', 'minecraft:long_weakness'),
    ('minecraft:awkward', 'minecraft:phantom_membrane', 'minecraft:slow_falling'),
    ('minecraft:slow_falling', 'minecraft:redstone', 'minecraft:long_slow_falling'),
]

# The container mixes: (from item, reagent, to item).
CONTAINER_MIXES = [
    ('minecraft:potion', 'minecraft:gunpowder', 'minecraft:splash_potion'),
    ('minecraft:splash_potion', 'minecraft:dragon_breath', 'minecraft:lingering_potion'),
]


def stack(item, potion=None, count=1):
    if potion:
        return '{id:"%s",count:%d,components:{"minecraft:potion_contents":{potion:"%s"}}}' % (
            item, count, potion)
    return '{id:"%s",count:%d}' % (item, count)


def potion(kind='potion', name='water', count=1):
    return '{id:"minecraft:%s",count:%d,components:{"minecraft:potion_contents":{potion:"minecraft:%s"}}}' % (
        kind, count, name)


def tick_case(ticks, slots, brew_time=0, fuel=0, ingredient='-'):
    filled = [slot if slot else '-' for slot in slots]
    return 'tick %d %s %d %d %s' % (
        ticks, ' '.join(filled), brew_time, fuel, ingredient)


def tick_cases():
    water = potion()
    awkward = potion(name='awkward')
    splash = potion('splash_potion')
    lingering = potion('lingering_potion')
    wart = stack('minecraft:nether_wart')
    wart2 = stack('minecraft:nether_wart', count=2)
    wart5 = stack('minecraft:nether_wart', count=5)
    wart21 = stack('minecraft:nether_wart', count=21)
    blaze = stack('minecraft:blaze_powder')
    blaze2 = '{id:"minecraft:blaze_powder",count:2}'
    redstone = stack('minecraft:redstone')
    gunpowder = stack('minecraft:gunpowder')
    empty_potion = stack('minecraft:potion')
    out = []
    # Brew-time countdown, with fuel already in the tank.
    for ticks in (0, 1, 2, 399, 400, 401, 402, 800):
        out.append(tick_case(ticks, [water, None, None, wart, None], fuel=20,
                             ingredient='minecraft:nether_wart'))
    # Blaze powder loads the tank (20 uses), then the brew runs.
    for ticks in (0, 1, 2, 399, 400, 401):
        out.append(tick_case(ticks, [water, None, None, wart, blaze],
                             ingredient='minecraft:nether_wart'))
    out.append(tick_case(1, [water, None, None, wart, blaze2],
                         ingredient='minecraft:nether_wart'))
    # Fuel loads even with nothing to brew.
    out.append(tick_case(1, [None, None, None, wart, blaze],
                         ingredient='minecraft:nether_wart'))
    out.append(tick_case(1, [None, None, None, None, blaze]))
    # Only the slots with a mix change; the no-mix bottle stays.
    for ticks in (1, 400, 401):
        out.append(tick_case(ticks, [water, awkward, water, wart, None], fuel=20,
                             ingredient='minecraft:nether_wart'))
    # The ingredient stack shrinks; the last one leaves an empty slot.
    for wart_stack, ticks in ((wart2, 400), (wart5, 400), (stack('minecraft:nether_wart'), 400)):
        out.append(tick_case(ticks, [water, None, None, wart_stack, None], fuel=20,
                             ingredient='minecraft:nether_wart'))
    # Changing the ingredient mid-brew clears the progress; finishing does not need fuel.
    out.append(tick_case(1, [water, None, None, redstone, None], brew_time=200, fuel=5,
                         ingredient='minecraft:nether_wart'))
    out.append(tick_case(1, [water, None, None, wart, None], brew_time=1, fuel=0,
                         ingredient='minecraft:nether_wart'))
    out.append(tick_case(1, [water, None, None, None, None], brew_time=1, fuel=5,
                         ingredient='minecraft:nether_wart'))
    # Splash and lingering potions keep their kind through the mix.
    out.append(tick_case(401, [splash, None, None, wart, None], fuel=20,
                         ingredient='minecraft:nether_wart'))
    out.append(tick_case(401, [lingering, None, None, wart, None], fuel=20,
                         ingredient='minecraft:nether_wart'))
    # A container mix keeps the potion: water potion + gunpowder -> splash water.
    out.append(tick_case(401, [water, None, None, gunpowder, None], fuel=20,
                         ingredient='minecraft:gunpowder'))
    # A potion with no contents plus gunpowder IS brewable (hasContainerMix
    # does not check contents), but mix leaves the bottle unchanged while the
    # ingredient is still consumed.
    out.append(tick_case(401, [empty_potion, None, None, gunpowder, None], fuel=20,
                         ingredient='minecraft:gunpowder'))
    # Twenty brews from one blaze powder's twenty uses; the 21st ingredient stays.
    out.append(tick_case(8001, [water, None, None, wart21, blaze],
                         ingredient='minecraft:nether_wart'))
    out.append(tick_case(8002, [water, None, None, wart21, blaze],
                         ingredient='minecraft:nether_wart'))
    # Bottle bits are set from the slots on the first tick.
    for slots in ([None, None, None], [water, None, None], [water, None, water],
                  [water, awkward, water]):
        out.append(tick_case(1, slots + [None, None], fuel=20))
    return out


def make_cases(build):
    resources = lab.LOCAL / 'horizon-client' / '26.2'
    catalog = json.loads((resources / 'items.json').read_text())
    potion_ids = sorted({p for mix in POTION_MIXES for p in (mix[0], mix[2])})
    reagents = sorted({mix[1] for mix in POTION_MIXES + CONTAINER_MIXES})
    cases = []
    # The baseline corpus: each vanilla mix's exact pair with each of the
    # three slots probed. --baseline reproduces it without the cross-product.
    for kind in POTION_ITEMS:
        for from_potion, reagent, _to in POTION_MIXES:
            inp = stack('minecraft:' + kind, from_potion)
            r = stack(reagent)
            for candidate in (inp, r, stack('minecraft:blaze_powder'),
                              stack('minecraft:stick')):
                cases.append(f"{inp} {r} {candidate}")
    for from_item, reagent, _to in CONTAINER_MIXES:
        for contents in ('minecraft:water', 'minecraft:awkward'):
            inp = stack(from_item, contents)
            r = stack(reagent)
            for candidate in (inp, r, stack('minecraft:blaze_powder'),
                              stack('minecraft:stick')):
                cases.append(f"{inp} {r} {candidate}")
    # A potion with no contents against the container reagents: hasMix says
    # brewable, mix says unchanged.
    for from_item, reagent, _to in CONTAINER_MIXES:
        inp = stack(from_item)
        cases.append(f"{inp} {stack(reagent)} {inp}")
    if '--baseline' not in sys.argv:
        # Every potion item and potion id against every reagent, plus a few
        # mismatches and empty sides.
        for item in POTION_ITEMS:
            for potion in potion_ids + ['minecraft:water', 'minecraft:awkward']:
                for reagent in reagents:
                    inp = stack('minecraft:' + item, potion)
                    cases.append(f"{inp} {stack(reagent)} {inp}")
                cases.append(f"{stack('minecraft:' + item, potion)} {stack('minecraft:stick')} -")
                cases.append(f"- {stack('minecraft:stick')} {stack('minecraft:' + item, potion)}")
        # Every item as a filter candidate, so the slot rules are compared on
        # the whole registry and not only on the mix inputs.
        for item in sorted(catalog):
            if item != 'minecraft:air':
                cases.append(f"- - {stack(item)}")
        for potion in potion_ids:
            for item in POTION_ITEMS:
                cases.append(f"- - {stack('minecraft:' + item, potion)}")
    if '--no-tick' not in sys.argv:
        cases += tick_cases()
    # Duplicate-free while keeping the first occurrence order.
    seen = set()
    unique = []
    for case in cases:
        if case not in seen:
            seen.add(case)
            unique.append(case)
    path = build / 'cases.txt'
    path.write_text(f'#resources {resources}\n' + ''.join(c + '\n' for c in unique))
    return path, len(unique)


if __name__ == '__main__':
    harness.differential(
        'brewing-parity', 'VanillaBrewingOracle', 'brewing_fixture', make_cases,
        'PotionBrewing mix matching and assemble over the vanilla mix table, the bottle, reagent '
        'and fuel slot filters, for every bundled mix, potion type/id and item candidate, plus the '
        'brewing-stand tick: the flat 400-tick brew, fuel uses and loading, ingredient shrink, '
        'outputs and bottle bits. --baseline leaves out the cross-product and whole-registry slot '
        'candidates; --no-tick leaves out the tick cases.',
        sources=[lab.ROOT / 'crates/horizon-client-render/src/brewing.rs',
                 lab.ROOT / 'crates/horizon-items/src/item_data.rs',
                 lab.ROOT / 'crates/horizon-items/src/items.rs'])
