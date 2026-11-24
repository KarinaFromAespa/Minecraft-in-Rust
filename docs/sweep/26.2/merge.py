#!/usr/bin/env python3
"""Merge the 26.2 sweep per-batch slices into pins/26.2/sweep-items.json.

Mirrors the 1.21.7 merged ledger shape (version, generated_from, note,
batches, items) with two additions: "totals" (the B6 tally convention makes
a single check total misleading without the split) and "dedup" (the fold
record, so nothing is silently dropped).

Whole-item duplicates are REMOVED from the merged items list (kept in their
slice files); the canonical item's evidence gains a [Merge note]. Per-oracle
member overlaps inside group items are annotated on both sides.
"""
import json
from collections import Counter

BASE = '/home/hatch/workspace/mcir-work/sweep-26.2/pins/26.2/sweep'

# Whole-item folds: duplicate id -> canonical id
WHOLE_FOLDS = {
    'B4-O16': 'B10-O03',  # VanillaBlockEntityRenderOracle (UvMapping)
    'B4-O18': 'B10-O05',  # VanillaRecipeBookScreenOracle (renderpearl, 1 of 5)
    'B5-O01': 'B1-O02',   # VanillaCauldronOracle (Prediction)
    'B5-O05': 'B1-O03',   # VanillaFallingBlockOracle (ChunkMap 13-vs-12)
    'B5-O06': 'B4-O06',   # VanillaDispenserOracle (onEquippedItemBroken)
    'B9-O03': 'B3-O04',   # LocatorBarClient (10e missing-symbol)
}

# Per-oracle member folds: (source group item, oracle) -> canonical item
MEMBER_FOLDS = [
    ('B3-O04', 'VanillaRegisteredEntityFluidOracle', 'B1-O01'),
    ('B3-O04', 'VanillaLivingFluidOracle', 'B1-O05'),
    ('B3-O04', 'VanillaTradingOracle', 'B4-O25'),
    ('B6-O04', 'VanillaWorldSpawnOracle', 'B3-O04'),
    ('B6-O04', 'VanillaSculkOracle', 'B4-O14'),
    ('B6-O06', 'VanillaFinalizeSpawnOracle', 'B3-O03'),
]

slices = {}
for n in range(13):
    with open(f'{BASE}/batch-{n:02d}.json') as f:
        slices[n] = json.load(f)

batches = {}
tot = Counter()
behav_tot = Counter()
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
    # B6 counts 54 oracle-status checks in its tally; behaviour-only
    # totals exclude them (28 pass / 26 fail).
    if n == 6:
        behav_tot['total'] += c['total'] - 54
        behav_tot['pass'] += c['pass'] - 28
        behav_tot['gap'] += c['gap'] - 26
    else:
        behav_tot['total'] += c['total']
        behav_tot['pass'] += c['pass']
        behav_tot['gap'] += c['gap']
    raw_items += len(d['items'])
# Batch 11's tally carries 2 accepted-gap items outside pass/gap.
tot['accepted_gap'] = 2
behav_tot['accepted_gap'] = 2

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
    'version': '26.2',
    'generated_from': ('docs/PARITY-SWEEP.md (branch parity-sweep); '
                       'per-batch slices in pins/26.2/sweep/batch-NN.json'),
    'note': (
        'Sweep ledger for 26.2, merged 2026-11-24 from the 13 per-batch slices. '
        'Same shape as the 1.21.7 merged ledger (pins/1.21.7/sweep-items.json): '
        'one item per defect found, with evidence, method, severity, status. '
        'Fixes are separate work, never on this branch; every fix field is null. '
        'ID convention: all ids are batch-prefixed (B<n>-Gxx behaviour/driver, '
        'B<n>-Oxx oracle, B<n>-Hxx needs-harness, O1-O8 Batch 0 repair groups). '
        'TALLY CONVENTION: Batch 6 counts oracle-status checks in its tally '
        '(79 = 25 behaviour + 54 oracle status), exactly as the 1.21.7 Batch 6 '
        'did (69 = 22 + 47); the other twelve batches do not. The "totals" '
        'block therefore gives both the as-tallied total and the behaviour-only '
        'total. STATUS MEANINGS: "gap" = demonstrated defect in 26.2, except '
        'B6-G03 (carried over from 1.21.7 UNVERIFIED — full aquifer audit found '
        'no divergence; B6-H arbitrates). "oracle-defect" = an oracle that fails '
        'javac against the 26.2 jars (Batch 0 measured table). "fixed" (B6-G01, '
        'B6-G02, B6-G04) means "1.21.7 defect withdrawn for 26.2 — verified '
        'absent", NOT a fix applied during this sweep. "closed" (15 Batch 9 '
        'items) means "1.21.7 defect re-checked and closed for 26.2". '
        '"accepted-gap" (B11-A01/A02) = known, accepted. BATCH 0 vs PER-BATCH: '
        'Batch 0 items O1-O8 are repair-group items (one per failure class); '
        'per-batch O-items are the same oracles recorded per driver family. '
        'Both are kept; the overlap is mapped in "dedup".batch0_overlap — it is '
        'not double-counting, it is two views of the same 129 failing oracles. '
        'ORACLE COVERAGE: the per-batch O-items name 84 distinct failing '
        'oracles; the remaining 45 failing oracles appear only in the Batch 0 '
        'group items (including VanillaProtocolClient, 32 errors, blocking 8 '
        'drivers, and VanillaModelBakingOracle — per-batch triage covered '
        'in-scope drivers only, not the full failing list). LANG CORRECTION: prepared en_us.json IS present and '
        'md5-identical to the 26.2 client jar file (8,123 keys) — the "lang '
        'absent" statements in the Batch 0/3/5/10 notes are wrong; only '
        'sounds.json is genuinely absent (B10-G02).'
    ),
    'batches': batches,
    'totals': {
        'checks_as_tallied': {
            'total': tot['total'], 'pass': tot['pass'], 'gap': tot['gap'],
            'accepted_gap': tot['accepted_gap'],
        },
        'checks_behaviour_only': {
            'total': behav_tot['total'], 'pass': behav_tot['pass'],
            'gap': behav_tot['gap'],
            'accepted_gap': behav_tot['accepted_gap'],
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
            'O1 (LocalPlayer ctor, 17 oracles)': 'Per-batch records: B3-O02 (5), '
                'B8-O01 (MarkerParticles), B9-O02 (TeamRules), B10-O04 (3 sound '
                'oracles). 7 oracles (AutoJump, Camera, GameLoop, HeldDrop, '
                'InteractionInput, LocalInput, SprintInput) appear only here.',
            'O2 (jsr305 client-libraries crash)': 'Latent driver defect; no '
                'per-batch item (test_model_baking_parity.py client_libraries '
                'omits jsr305).',
            'O3 (fuelValues + 3 missing overrides)': 'Per-batch records: B3-O01 '
                '(3 fuelValues oracles), B4-O15 (BlockEntityAnim), B5-O07 '
                '(SectionMesh bake), B6-O05 (DecorationRegion/FeatureCatalog lookup).',
            'O4 (densityfunction, 9 oracles)': 'Per-batch record: B6-O01 (8 of 9; '
                'VanillaFloatLogOracle appears only here).',
            'O5 (other 26.3-only packages)': 'Per-batch records: B10-O05 '
                '(renderpearl, 5 of 7; SpriteAnimation + Translucency only here), '
                'B6-O02 (material/rule, all 3). TabList (authlib.services) and '
                'TickScript (lwjgl.sdl) appear only here.',
            'O6 (129 fail / 147 drivers blocked)': 'Headline roll-up; every '
                'per-batch oracle item is a subset of these 129.',
            'O7 (StructureDensity unrepairable)': 'Also a member of B6-O01; '
                'kept as its own item because it is unrepairable by design.',
            'O8 (runtime layer unmeasured)': 'No per-batch item; only '
                'command_parse was run end-to-end (0 mismatches, 103,743 parses).',
        },
    },
    'items': items,
}

out = '/home/hatch/workspace/mcir-work/sweep-26.2/pins/26.2/sweep-items.json'
with open(out, 'w') as f:
    json.dump(merged, f, indent=1, sort_keys=True)
print('wrote', out)
print('checks as-tallied:', dict(tot))
print('checks behaviour-only:', dict(behav_tot))
print('items raw:', raw_items, '| unique:', len(items), '| live:', len(live),
      '| live major:', len(live_major))
print('by status:', dict(st))
print('by severity:', dict(sev))
print('live majors:', sorted(i['id'] for i in live_major))
