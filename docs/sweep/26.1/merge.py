#!/usr/bin/env python3
"""Merge the 26.1 sweep per-batch slices into pins/26.1/sweep-items.json.

Mirrors the 26.2 merged ledger shape (version, generated_from, note,
batches, items, totals, dedup) with one convention change: TALLY
CONVENTION — every batch counts behaviour checks only; NO batch counts
oracle-status checks in its tally (the 26.2 run's Batch 6 did, and the
split totals it forced were the one wart in its ledger). The "totals"
block therefore gives a single check total, plus the behaviour-only row
kept for cross-version comparison (identical here by construction).

Whole-item duplicates are REMOVED from the merged items list (kept in their
slice files); the canonical item's evidence gains a [Merge note]. Per-oracle
member overlaps inside group items are annotated on both sides.

WHOLE_FOLDS / MEMBER_FOLDS are filled in at merge time from the actual
duplicate record — do not pre-fill from the 26.2 run's folds.
"""
import json
from collections import Counter

BASE = '/home/hatch/workspace/mcir-work/sweep-26.1/pins/26.1/sweep'

# Whole-item folds: duplicate id -> canonical id (filled at merge time)
WHOLE_FOLDS = {
}

# Per-oracle member folds: (source group item, oracle) -> canonical item
MEMBER_FOLDS = [
]

slices = {}
for n in range(13):
    with open(f'{BASE}/batch-{n:02d}.json') as f:
        slices[n] = json.load(f)

batches = {}
tot = Counter()
raw_items = 0
for n, d in slices.items():
    batches[str(n)] = {
        'name': d.get('name', f'Batch {n}'),
        'date': d.get('date', '2026-11-24'),
        'checks': d.get('checks', {}),
        'status': 'complete',
    }
    c = d['checks']
    tot['total'] += c['total']
    tot['pass'] += c['pass']
    tot['gap'] += c['gap']
    tot['accepted_gap'] += c.get('accepted_gap', 0)
    raw_items += len(d['items'])

# Collect items, normalizing to the 1.21.7 merged shape.
items = []
absorbed = {}   # canonical id -> list of folded duplicate ids
lost = {}       # group id -> list of (oracle, canonical id)
for src, oracle, canon in MEMBER_FOLDS:
    lost.setdefault(src, []).append((oracle, canon))
for dup, canon in WHOLE_FOLDS.items():
    absorbed.setdefault(canon, []).append(dup)

for n in range(13):
    for it in slices[n]['items']:
        if it['id'] in WHOLE_FOLDS:
            continue  # folded into its canonical item
        title = it.get('title') or it.get('name', '')
        ev = it.get('evidence', '')
        notes = []
        if it['id'] in absorbed:
            notes.append(
                'Absorbs duplicate item(s) %s (same oracle defect, '
                'recorded again in a later batch; folded here at merge).'
                % ', '.join(sorted(absorbed[it['id']])))
        if it['id'] in lost:
            notes.append(
                'Member oracle(s) folded into earlier canonical items at '
                'merge: %s. The remaining oracles in this group are first '
                'records here.'
                % '; '.join(f'{o} -> {c}' for o, c in lost[it['id']]))
        # Canonical items that also absorbed member folds:
        member_absorbed = [f'{o} (from {s})' for s, o, c in MEMBER_FOLDS
                           if c == it['id']]
        if member_absorbed:
            notes.append(
                'Absorbs member oracle(s) %s, also recorded inside a later '
                'batch group item; folded here at merge.'
                % ', '.join(member_absorbed))
        if notes:
            ev = ev + '\n\n[Merge note: ' + ' '.join(notes) + ']'
        items.append({
            'evidence': ev,
            'fix': None,
            'id': it['id'],
            'method': it.get('method', ''),
            'severity': it.get('severity', 'minor'),
            'status': it.get('status', 'gap'),
            'title': title,
        })

sev = Counter(i['severity'] for i in items)
st = Counter(i['status'] for i in items)
live = [i for i in items if i['status'] in ('gap', 'oracle-defect', 'needs-harness')]
live_major = [i for i in live if i['severity'] == 'major']

merged = {
    'version': '26.1',
    'generated_from': ('docs/PARITY-SWEEP.md (branch parity-sweep); '
                       'per-batch slices in pins/26.1/sweep/batch-NN.json'),
    'note': (
        'Sweep ledger for 26.1, merged 2026-11-24 from the 13 per-batch slices. '
        'Same shape as the 1.21.7 merged ledger (pins/1.21.7/sweep-items.json): '
        'one item per defect found, with evidence, method, severity, status. '
        'Fixes are separate work, never on this branch; every fix field is null. '
        'ID convention: all ids are batch-prefixed (B<n>-Gxx behaviour/driver, '
        'B<n>-Oxx oracle, B<n>-Hxx needs-harness, O1-O8 Batch 0 repair groups). '
        'TALLY CONVENTION: every batch counts behaviour checks only; no batch '
        'counts oracle-status checks in its tally (unlike the 26.2 run, where '
        'Batch 6 did). The "totals" block therefore gives one check total. '
        'STATUS MEANINGS: "gap" = demonstrated defect in 26.1. '
        '"oracle-defect" = an oracle that fails javac against the 26.1 jars '
        '(Batch 0 measured table). "fixed" means "1.21.7 defect withdrawn for '
        '26.1 — verified absent", NOT a fix applied during this sweep. '
        '"closed" means "1.21.7 defect re-checked and closed for 26.1". '
        '"accepted-gap" = known, accepted. BATCH 0 vs PER-BATCH: Batch 0 items '
        'O1-O8 are repair-group items (one per failure class); per-batch '
        'O-items are the same oracles recorded per driver family. Both are '
        'kept; the overlap is mapped in "dedup".batch0_overlap — it is not '
        'double-counting, it is two views of the same failing oracles. '
        'ORACLE COVERAGE: [filled at merge time — distinct failing oracles '
        'named by per-batch O-items vs those appearing only in Batch 0 groups].'
    ),
    'batches': batches,
    'totals': {
        'checks': {
            'total': tot['total'], 'pass': tot['pass'], 'gap': tot['gap'],
            'accepted_gap': tot['accepted_gap'],
        },
        'checks_behaviour_only': {
            'total': tot['total'], 'pass': tot['pass'], 'gap': tot['gap'],
            'accepted_gap': tot['accepted_gap'],
        },
        'items_raw': raw_items,
        'items_unique': len(items),
        'items_live': len(live),
        'items_live_major': len(live_major),
        'items_by_status': dict(st),
        'items_by_severity': dict(sev),
        'live_major_ids': sorted(i['id'] for i in live_major),
    },
    'dedup': {
        'whole_item_folds': [
            {'duplicate': d, 'canonical': c,
             'note': 'Same oracle defect recorded in two batches; '
                     'duplicate removed from the merged list, kept in its slice.'}
            for d, c in sorted(WHOLE_FOLDS.items())
        ],
        'oracle_member_folds': [
            {'from_group': s, 'oracle': o, 'canonical': c,
             'note': 'Oracle also recorded inside a later batch group item; '
                     'the group item stays, this member folds into the '
                     'earlier canonical item.'}
            for s, o, c in MEMBER_FOLDS
        ],
        'batch0_overlap': {
            # Filled at merge time from the Batch 0 groups + per-batch records.
        },
    },
    'items': items,
}

out = '/home/hatch/workspace/mcir-work/sweep-26.1/pins/26.1/sweep-items.json'
with open(out, 'w') as f:
    json.dump(merged, f, indent=1, sort_keys=True)
print('wrote', out)
print('checks:', dict(tot))
print('items raw:', raw_items, '| unique:', len(items), '| live:', len(live),
      '| live major:', len(live_major))
print('by status:', dict(st))
print('by severity:', dict(sev))
print('live majors:', sorted(i['id'] for i in live_major))
