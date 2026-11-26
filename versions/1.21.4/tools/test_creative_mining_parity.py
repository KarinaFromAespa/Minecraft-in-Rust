#!/usr/bin/env python3
"""Compare Creative mining eligibility for every item and tool overrides."""
import json

try:
    from . import lab, oracle_harness as harness
except ImportError:
    import lab, oracle_harness as harness


def make_input(build):
    resources = lab.LOCAL / 'horizon-client' / lab.pinned_version()
    items = json.loads((resources / 'items.json').read_text())
    rows = [dict(item=item, creative=creative, count=count, tool=tool)
            for item in sorted(items)
            for creative in (False, True)
            for count in (0, 1)
            for tool in ('default', 'allow', 'deny', 'implicit')]
    path = build / 'inputs.jsonl'
    path.write_text(''.join(json.dumps(row) + '\n' for row in rows))
    return path, len(rows)


def main():
    harness.differential(
        'rust-creative-mining-parity', 'VanillaCreativeMiningOracle',
        'creative_mining_probe', make_input,
        '1.21.4 has no `Item.canDestroyBlock`: a creative player destroys any block with any held '
        'item, whatever the stack tool component says. Block destruction and network '
        'prediction call this predicate; end-to-end packet handling is not covered here.',
        sources=[lab.ROOT / 'crates/horizon-items/src/items.rs'])


if __name__ == '__main__':
    main()
