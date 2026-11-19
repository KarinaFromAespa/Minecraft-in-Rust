#!/usr/bin/env python3
"""Compare World Options packet payloads with the pinned vanilla codecs."""
try:
    from . import lab, oracle_harness as harness
except ImportError:
    import lab, oracle_harness as harness


def cases(build):
    # 1.21.11 has no game-rule packets: difficulty and lock only.
    lines = [f'difficulty\t{i}' for i in range(4)] + ['lock\tfalse', 'lock\ttrue']
    path = build / 'cases.txt'
    path.write_text('\n'.join(lines) + '\n')
    return path, len(lines)


if __name__ == '__main__':
    harness.differential(
        'world-options-parity', 'VanillaWorldOptionsOracle', 'world_options_fixture', cases,
        'ChangeDifficulty and LockDifficulty payloads',
        sources=[lab.ROOT / 'crates/horizon-protocol/src/packets/play.rs'], expected_lines=6)
