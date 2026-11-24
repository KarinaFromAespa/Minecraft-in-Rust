#!/usr/bin/env python3
"""Merge the 26.1.1 sweep per-batch slices into pins/26.1.1/sweep-items.json.

Mirrors the 26.1 merged ledger shape (version, generated_from, note,
batches, items, totals, dedup). TALLY: the "totals" block gives both the
as-tallied total (every batch's slice tally summed as delivered) and the
behaviour-only total. NON_BEHAVIOUR (below) is filled at merge time from
the actual non-behaviour checks the batches counted in their as-tallied
tallies (26.2 precedent: Batch 4 driver-status, Batch 3 driver-code-
defect, Batch 9 oracle/harness checks).

Whole-item duplicates are REMOVED from the merged items list (kept in their
slice files); the canonical item's evidence gains a [Merge note]. Per-oracle
member overlaps inside group items are annotated on both sides.

WHOLE_FOLDS / MEMBER_FOLDS are filled in at merge time from the actual
duplicate record — do not pre-fill from the 26.2 run's folds.
"""
import json
from collections import Counter

BASE = '/home/hatch/workspace/mcir-work/sweep-26.1.1/pins/26.1.1/sweep'

# Whole-item folds: duplicate id -> canonical id (filled at merge time)
# Canonical = family owner / earlier record (26.2 model). 31 folds.
WHOLE_FOLDS = {
    # Redstone wire / dirt path / dripstone / splash water / mud (1.21.7 lineage; 26.1 precedents)
    'B11-G02': 'B2-G01',   # redstone wire
    'B11-G03': 'B5-G03',   # dirt path
    'B11-G04': 'B1-G01',   # dripstone 3-way, canonical B1-G01 (3 dead causes)
    'B5-G13': 'B1-G01',    # dripstone 3-way
    'B1-G03': 'B11-G05',   # splash water 3-way, canonical B11-G05 (1.21.7 lineage)
    'B7-G10': 'B11-G05',   # splash water 3-way
    'B7-G11': 'B5-G19',    # mud tag spelling
    'B4-G18': 'B5-G19',    # mud 3-way
    # Cauldron / FallingBlock / LavaCauldronItem (26.2 precedents)
    'B5-O01': 'B1-O02',    # Cauldron 3-way
    'B11-O01': 'B1-O02',   # Cauldron 3-way
    'B5-O05': 'B1-O03',    # FallingBlock
    'B11-O02': 'B1-O07',   # LavaCauldronItem 3-way
    'B7-O07': 'B1-O07',    # LavaCauldronItem 3-way
    'B3-O08': 'B1-O06',    # EntityFluid single-oracle group -> B1 singleton
    # Dispenser / BlockUpdate / FaceSupport
    'B5-O06': 'B4-O06',    # Dispenser 3-way, canonical B4-O06 (26.2 precedent)
    'B7-O01': 'B4-O06',    # Dispenser 3-way
    'B7-O02': 'B2-O01',    # BlockUpdate, 32 errors
    'B5-O09': 'B2-O03',    # FaceSupport
    # Render / UI family singletons -> B10 group items (26.2 precedents)
    'B7-O09': 'B10-O03',   # ItemModel singleton -> render family
    'B5-O07': 'B10-O03',   # SectionMesh singleton -> render family
    'B5-O08': 'B10-O03',   # BreakingModel singleton -> render family
    'B4-O16': 'B10-O03',   # BlockEntityRender (26.2 precedent exactly)
    'B4-O18': 'B10-O05',   # RecipeBookScreen (26.2 precedent exactly)
    'B4-O08': 'B10-O05',   # MenuOracle singleton -> UI family
    # Containers / trading / spawner / loot
    'B7-O10': 'B4-O21',    # VehicleContainer
    'B4-G20': 'B7-G17',    # spawner setEntityData (3-way with B3-G39)
    'B3-G39': 'B7-G17',    # spawner setEntityData, trial arm (NEW vs 26.1)
    'B3-G24': 'B4-G24',    # villager level-up (B4 owns trading)
    'B4-G26': 'B12-G01',   # loot engine (B12 canonical, per-table impact counts)
    # New 26.1.1 duplicates (no 26.1 record — the 26.1 tree had no composter
    # defect and no dispenser-spawn-egg defect)
    'B4-G30': 'B5-G22',    # composter (B5 canonical: fuller evidence, all paths)
    'B4-G35': 'B7-G18',    # dispenser spawn egg on null spawn (B7 owns spawn-egg items)
}

# Per-oracle member folds: (source group item, oracle) -> canonical item
MEMBER_FOLDS = [
    ('B10-O03', 'VanillaBlockEntityAnimOracle', 'B4-O15'),
    ('B6-O04', 'VanillaSculkOracle', 'B4-O14'),
    ('B6-O06', 'VanillaNaturalSpawnOracle', 'B4-O23'),
    ('B6-O06', 'VanillaFinalizeSpawnOracle', 'B3-O03'),
    ('B6-O06', 'VanillaSpawnRulesOracle', 'B3-O03'),  # NEW vs 26.1: SpawnRules in both groups
    ('B6-O01', 'VanillaWorldSpawnOracle', 'B3-O05'),
    ('B3-O03', 'VanillaNaturalSpawnOracle', 'B4-O23'),
    ('B3-O05', 'VanillaRegisteredEntityFluidOracle', 'B1-O01'),
    ('B3-O05', 'VanillaLivingFluidOracle', 'B1-O05'),
    ('B3-O05', 'VanillaTradingOracle', 'B4-O25'),
    ('B3-O03', 'VanillaArrowOracle', 'B7-O03'),
    ('B3-O03', 'VanillaBoatRiderOracle', 'B7-O04'),
    ('B3-O03', 'VanillaClientArrowOracle', 'B7-O05'),
    ('B3-O03', 'VanillaBlockingItemDamageOracle', 'B7-O17'),
    ('B3-O05', 'VanillaVehicleContainerOracle', 'B4-O21'),
    ('B3-O05', 'VanillaPlayerInventoryOracle', 'B7-O27'),
    ('B3-O05', 'KnockbackClient', 'B7-O11'),
    ('B3-O07', 'VanillaHelmetDamageOracle', 'B7-O30'),
    ('B3-O04', 'VanillaBoatAuthorityOracle', 'B7-O08'),
    ('B3-O04', 'VanillaShulkerSpaceOracle', 'B4-O22'),
    ('B3-O02', 'VanillaEntitySoundPacketOracle', 'B10-O04'),
    ('B3-O05', 'LocatorBarClient', 'B9-O03'),
]

slices = {}
for n in range(13):
    with open(f'{BASE}/batch-{n:02d}.json') as f:
        slices[n] = json.load(f)

batches = {}
tot = Counter()
# Behaviour-only restatement: remove the non-behaviour checks the
# batches counted in their as-tallied tallies (each per a 26.2 precedent,
# accepted by the coordinator): B4 37 driver-status checks (17 pass,
# 20 gap — passing-oracle + oracle-blocked driver rows; the no-oracle
# driver rows are behaviour checks by assertion read), B3 3
# driver-code-defect checks (0 pass, 3 gap), B9 4 oracle/harness checks
# (1 pass, 3 gap — H01 is FIXED for 26.1.1, so it counts as a pass).
# Total removed: 44 checks (18 pass, 26 gap).
NON_BEHAVIOUR = {4: (17, 20), 3: (0, 3), 9: (1, 3)}
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
    tot['accepted_gap'] += c.get('accepted_gap', 0)
    bp, bg = NON_BEHAVIOUR.get(n, (0, 0))
    behav_tot['total'] += c['total'] - bp - bg
    behav_tot['pass'] += c['pass'] - bp
    behav_tot['gap'] += c['gap'] - bg
    behav_tot['accepted_gap'] += c.get('accepted_gap', 0)
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
                'Absorbs duplicate item(s) %s (same defect, '
                'recorded again in another batch; folded here at merge).'
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
                'Absorbs member oracle(s) %s, also recorded inside another '
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
    'version': '26.1.1',
    'generated_from': ('docs/PARITY-SWEEP.md (branch parity-sweep); '
                       'per-batch slices in pins/26.1.1/sweep/batch-NN.json'),
    'note': (
        'Sweep ledger for 26.1.1, merged 2026-11-24 from the 13 per-batch slices. '
        'Same shape as the 26.1 merged ledger (pins/26.1/sweep-items.json): '
        'one item per defect found, with evidence, method, severity, status. '
        'Fixes are separate work, never on this branch; every fix field is null. '
        'ID convention: all ids are batch-prefixed (B<n>-Gxx behaviour/driver, '
        'B<n>-Oxx oracle, B<n>-Hxx needs-harness, O1-O9 Batch 0 repair groups). '
        'TALLY CONVENTION: the "totals" block gives both the as-tallied total '
        '(every batch slice tally summed as delivered) and the behaviour-only '
        'total. Three batches count non-behaviour checks in their as-tallied '
        'tallies, each per a 26.2 precedent the coordinator accepted: Batch 4 '
        'counts 37 driver-status checks (17 pass, 20 gap), Batch 3 counts 3 '
        'driver-code-defect checks (0 pass, 3 gap), Batch 9 counts 3 oracle '
        'items + 1 harness blocker as checks (1 pass, 3 gap — H01 is fixed '
        'for 26.1.1, so it counts as a pass). The behaviour-only total '
        'removes all 44 (18 pass, 26 gap). No other batch counts '
        'oracle-status checks. '
        'STATUS MEANINGS: "gap" = demonstrated defect in 26.1.1. '
        '"oracle-defect" = an oracle that fails javac against the 26.1.1 jars '
        '(Batch 0 measured table). "fixed" means "26.1 defect withdrawn for '
        '26.1.1 — verified absent", NOT a fix applied during this sweep. '
        '"closed" means "26.1 defect re-checked and closed for 26.1.1". '
        '"accepted-gap" = known, accepted. BATCH 0 vs PER-BATCH: Batch 0 items '
        'O1-O9 are repair-group items (one per failure class); per-batch '
        'O-items are the same oracles recorded per driver family. Both are '
        'kept; the overlap is mapped in "dedup".batch0_overlap — it is not '
        'double-counting, it is two views of the same 202 failing files. '
        'ORACLE COVERAGE: the per-batch O-items name 162 distinct failing '
        'files (156 Vanilla*Oracle + 6 harness/client files); the remaining 40 '
        'failing files appear only in the Batch 0 group items (including '
        'VanillaProtocolClient, 34 errors, blocking 8 drivers, and '
        'VanillaTickScript, 19 errors — per-batch triage covered in-scope '
        'drivers only, not the full failing list). B6 short oracle names map '
        '1:1 to Vanilla*Oracle; B3-O04 EntityTranscript/RideTranscript are '
        'driver/transcript files, not oracles. MEASUREMENT: command_parse ran '
        'end-to-end for 26.1.1 (Batch 9 §2 — H01 fixed, the probe compiles); '
        'it is the only end-to-end driver run in this sweep. The loot_fill '
        'example was NOT run for 26.1.1 (Batch 12 rests on code trace + '
        'census; the 26.1 live run covers the identical code shape). Batch '
        '9\'s 18 other e2e drivers are unrun (command_exec fixture blocker). '
        'LANG: prepared en_us.json IS present and md5-identical to the 26.1.1 '
        'client jar file (7,886 keys); only sounds.json is genuinely absent '
        '(B10-G02).'
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
             'note': 'Same defect recorded in two batches; '
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
            'O1 (LocalPlayer ctor, 20 oracles)': 'Per-batch records: B3-O02 '
                '(5), B10-O04 (3 sound oracles), B3-O03, B7-O04, B7-O21, '
                'B7-O22, B7-O28, B8-O01, B9-O02, B10-O03, B10-O05 (1 each). '
                '5 oracles (AutoJump, Camera, GameLoop, LocalInput, '
                'SprintInput) appear only here.',
            'O2 (jsr305 client-libraries crash)': 'Latent driver defect; no '
                'per-batch item (test_model_baking_parity.py client_libraries '
                'omits jsr305).',
            'O3 (fuelValues + missing overrides, 7 oracles)': 'Per-batch '
                'records: B3-O01 (3 fuelValues oracles), B6-O05 (2, '
                'DecorationRegion/FeatureCatalog lookup), B10-O03 (2), '
                'B4-O15 (BlockEntityAnim), B4-O16 (1, folds into B10-O03). '
                'None appear only here.',
            'O4 (densityfunction, 17 oracles)': 'Per-batch records: B6-O01 '
                '(10), B6-O02 (5), B6-O07 (4), B3-O05 (1), B6-O04 (1). None '
                'appear only here.',
            'O5 (EntityTypes holder move, 31 oracles)': 'Per-batch records: '
                'B3-O04 (11), B3-O03 (5), B3-O05 (4), B3-O02 (3), B10-O03 (2), '
                'B6-O06 (2), and 14 singletons (B4-O06, B4-O08, B4-O15, '
                'B4-O22, B4-O23, B4-O25, B5-O06, B7-O01, B7-O04, B7-O08, '
                'B7-O09, B7-O11, B9-O03, B10-O05). 7 oracles (ProtocolClient, '
                'RegistryFallRay, SceneMotion, SourceSilence, SprintInput, '
                'WetFire, WorldRay) appear only here.',
            'O6 (other 26.3-only packages, 17 oracles)': 'Per-batch records: '
                'B10-O05 (renderpearl, 8), B6-O02 (material/rule, 4), B3-O06 '
                '(advancements, 2), B4-O18 (1), B6-O01 (1), B6-O07 (1). 2 '
                'oracles (ComponentWireCapture, TickScript) appear only here.',
            'O7 (202 fail / 231 drivers blocked)': 'Headline roll-up; every '
                'per-batch oracle item is a subset of these 202. The 14 '
                'oracles named in its evidence (top blockers) have per-batch '
                'records except 7 (ComponentWireCapture, OfflineLanHost, '
                'CollisionContext, LootFill, Mover, RegistryCollisionScan, '
                'SceneMotion). 19 of the 40 Batch-0-only failing files belong '
                'to no group evidence list (the long tail).',
            'O8 (runtime layer unmeasured)': 'No per-batch item. Partially '
                'measured now: command_parse ran end-to-end (Batch 9 §2); the '
                'remaining runtime layer (other drivers) is still unmeasured.',
            'O9 (re-ported 26.3-form oracle sources, 17 files)': 'Per-batch '
                'records: B6-O04 (5), B6-O03 (2), B6-O05 (2), B4-O04, B4-O05, '
                'B5-O04, B6-O06, B10-O03 (1 each). 3 oracles (BlockTransformer, '
                'FillLayer, SteppedInterpolation) appear only here.',
        },
    },
    'items': items,
}

out = '/home/hatch/workspace/mcir-work/sweep-26.1.1/pins/26.1.1/sweep-items.json'
with open(out, 'w') as f:
    json.dump(merged, f, indent=1, sort_keys=True)
print('wrote', out)
print('checks:', dict(tot))
print('items raw:', raw_items, '| unique:', len(items), '| live:', len(live),
      '| live major:', len(live_major))
print('by status:', dict(st))
print('by severity:', dict(sev))
print('live majors:', sorted(i['id'] for i in live_major))
