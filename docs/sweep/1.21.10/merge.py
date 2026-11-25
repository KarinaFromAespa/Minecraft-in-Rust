#!/usr/bin/env python3
"""Merge the 1.21.10 sweep per-batch slices into pins/1.21.10/sweep-items.json.

Mirrors the 26.1.2 merged ledger shape (version, generated_from, note,
batches, items, totals, dedup). TALLY: the "totals" block gives both the
as-tallied total (every batch's slice tally summed as delivered, with the
B5 correction below) and the behaviour-only total. NON_BEHAVIOUR (below)
is filled at merge time from the actual non-behaviour checks the batches
counted in their as-tallied tallies (26.2 precedent: Batch 4 driver-status,
Batch 3 driver-code-defect, Batch 9 oracle/harness checks).

Whole-item duplicates are REMOVED from the merged items list (kept in their
slice files); the canonical item's evidence gains a [Merge note]. Per-oracle
member overlaps inside group items are annotated on both sides.

COORDINATOR CORRECTIONS (verified by the coordinator against the tree,
the jars, and the Batch 0 measured table — see sweep-1.21.10-batches/
merge-notes.md):
1. B5 tally: 101 pass / 27 gap as delivered -> 102 / 26. The port honours
   doFireTick via gamerules::fire_radius -> fire_spread_radius_around_player
   -> can_spread_fire_around (tick.rs:635, fluids.rs:748). B1-G11 stays FIXED.
2. B7-O11: fixed -> oracle-defect. B7 claimed KnockbackClient fixed (no
   ServerboundPunchPacket ref, all 49 imports exist); the B0 table says it
   FAILS on all three classpaths (7 no-override errors, first error line
   362). The 26.1.2 cause is gone but a new cause replaces it.
3. B5-O03: oracle-defect -> fixed. B5 claimed VanillaPickBlockDataOracle
   fails because TypedEntityData has no loadInto; the B0 table says it
   PASSES on all three, and the coordinator's own javap confirmed
   TypedEntityData HAS both loadInto overloads in 1.21.10's jars.

MINTED AT MERGE: B12-G09 (frogspawn can never survive) from B12-G03's
evidence — B12-G03 folds into the six B5 per-block items, and frogspawn
is its seventh block, covered nowhere else.

WHOLE_FOLDS were verified fresh against the 1.21.10 slices: every
participant exists, every pair is live, and the duplicate/canonical
evidence was compared (splash-water 3-way, mud, spawner, dripstone,
supports_* census vs per-block items, cauldron 3-way, dispenser 3-way,
lava-cauldron 3-way, render/UI singletons -> B10 families all confirmed).
"""
import json
from collections import Counter

BASE = '/home/hatch/workspace/mcir-work/sweep-1.21.10/pins/1.21.10/sweep'

# Whole-item folds: duplicate id -> canonical id. 24 folds.
WHOLE_FOLDS = {
    # Splash water 3-way, canonical B1-G03 (fluids batch owns it; B7 asked
    # to fold, B11's title says "overlaps B1-G03").
    'B7-G10': 'B1-G03',
    'B11-G05': 'B1-G03',
    # Mud tag spelling (B7 asked to fold; same four sites).
    'B7-G11': 'B5-G19',
    # Spawner setEntityData (B7 asked to fold; same defect, both arms).
    'B7-G17': 'B3-G39',
    # Dripstone tag half: B11-G04 is a subset of B5-G13's evidence.
    # B1-G01 stays separate (fluid-transfer/cauldron side + blockToGrowOn).
    'B11-G04': 'B5-G13',
    # supports_* survival census -> the six B5 per-block items (canonical
    # B5-G37, first of the set; frogspawn minted separately as B12-G09).
    'B12-G03': 'B5-G37',
    # Cauldron 3-way, canonical B1-O02.
    'B5-O01': 'B1-O02',
    'B11-O01': 'B1-O02',
    # FallingBlock.
    'B5-O05': 'B1-O03',
    # LavaCauldronItem 3-way, canonical B1-O07.
    'B7-O07': 'B1-O07',
    'B11-O02': 'B1-O07',
    # EntityFluid single-oracle group -> B1 singleton.
    'B3-O08': 'B1-O06',
    # Dispenser 3-way, canonical B4-O06 (26.2 precedent).
    'B5-O06': 'B4-O06',
    'B7-O01': 'B4-O06',
    # BlockUpdate, 63 errors.
    'B7-O02': 'B2-O01',
    # FaceSupport.
    'B5-O09': 'B2-O06',
    # VehicleContainer.
    'B7-O10': 'B4-O21',
    # EnchantDurability cascade (both cascade on BlockUpdate).
    'B7-O29': 'B4-O19',
    # Render / UI family singletons -> B10 group items (26.2 precedents).
    'B7-O09': 'B10-O03',   # ItemModel singleton -> render family
    'B5-O07': 'B10-O03',   # SectionMesh singleton -> render family
    'B5-O08': 'B10-O03',   # BreakingModel singleton -> render family
    'B4-O16': 'B10-O03',   # BlockEntityRender (26.2 precedent exactly)
    'B4-O18': 'B10-O05',   # RecipeBookScreen (26.2 precedent exactly)
    'B4-O08': 'B10-O05',   # MenuOracle singleton -> UI family
}

# Custom merge notes for canonicals whose fold needs more than the default
# "absorbs duplicate" text.
FOLD_NOTES = {
    'B5-G37': ('Absorbs B12-G03 (same defect, recorded as one census item '
               'covering seven blocks; the other five blocks are B5-G38-'
               'G42; the seventh block, frogspawn, is minted separately as '
               'B12-G09).'),
}

# Per-oracle member folds: (source group item, oracle) -> canonical item.
# Cascade/dependency mentions are NOT members (e.g. BlockUpdate named in
# B4-O03/B4-O19/B7-O29/B7-O30/B3-O07 as the cascade source; FaceSupport
# named in B2-O07/B3-O07 as the cascade source; RegistryFluid named in
# B1-O04 as a shared-symbol cross-reference) — those get no fold.
MEMBER_FOLDS = [
    ('B3-O11', 'VanillaActiveEffectTickOracle', 'B3-O01'),
    ('B3-O11', 'VanillaAttributeUpdatesOracle', 'B3-O01'),
    ('B3-O10', 'VanillaArrowOracle', 'B7-O03'),
    ('B10-O03', 'VanillaBlockEntityAnimOracle', 'B4-O15'),
    ('B10-O05', 'VanillaContainerScreenOracle', 'B4-O17'),
    ('B3-O10', 'VanillaFluidBoxOracle', 'B1-O08'),
    ('B3-O10', 'VanillaEffectSyncOracle', 'B8-O07'),
    ('B3-O02', 'VanillaEntitySoundPacketOracle', 'B10-O04'),
    ('B3-O10', 'VanillaEntitySoundPacketOracle', 'B10-O04'),
    ('B6-O02', 'VanillaMaterialSurfaceOracle', 'B6-O01'),
    ('B3-O09', 'VanillaNaturalSpawnOracle', 'B4-O23'),
    ('B6-O06', 'VanillaNaturalSpawnOracle', 'B4-O23'),
    ('B6-O08', 'VanillaNaturalSpawnOracle', 'B4-O23'),
    ('B10-O05', 'VanillaRecipeBookOracle', 'B4-O09'),
    ('B10-O05', 'VanillaPlaceRecipeOracle', 'B4-O10'),
    ('B10-O05', 'VanillaSimpleContainerOracle', 'B4-O11'),
    ('B6-O06', 'VanillaSpawnRulesOracle', 'B3-O10'),
    ('B6-O08', 'VanillaSpawnRulesOracle', 'B3-O10'),
    ('B3-O05', 'VanillaTradeComponentsOracle', 'B4-O27'),
]

# Coordinator corrections: item id -> (new status, new title, note).
CORRECTIONS = {
    'B7-O11': (
        'oracle-defect',
        'KnockbackClient fails javac: 7 no-override errors (ServerboundPunchPacket cause gone, new cause replaces it)',
        'Coordinator correction: B7 recorded this as fixed (no '
        'ServerboundPunchPacket reference, all 49 imports exist), but the '
        'Batch 0 measured table says KnockbackClient FAILS on all three '
        'classpaths (7 no-override errors, first error line 362). The '
        '26.1.2 cause is gone but a new cause replaces it. The '
        'item_use_order driver does NOT flip to runnable.',
    ),
    'B5-O03': (
        'fixed',
        '[FIXED] VanillaPickBlockDataOracle compiles for 1.21.10 (TypedEntityData has both loadInto overloads)',
        'Coordinator correction: B5 recorded this as an oracle defect '
        '(TypedEntityData has no loadInto), but the Batch 0 measured table '
        'says VanillaPickBlockDataOracle PASSES on all three classpaths, '
        'and the coordinator\'s own javap confirmed TypedEntityData HAS '
        'both loadInto overloads in 1.21.10\'s jars. B5\'s javap read was '
        'wrong.',
    ),
}

# B5 tally correction: pass 101 -> 102, gap 27 -> 26 (doFireTick check
# passes; the port honours it via gamerules::fire_radius).
TALLY_CORRECTIONS = {5: {'pass': 102, 'gap': 26}}

# Items minted at merge from folded-item evidence.
MINTED = [
    {
        'evidence': (
            'Minted at merge from B12-G03\'s evidence (the census item '
            'folds into the six B5 per-block items; frogspawn is its '
            'seventh block, covered nowhere else). Vanilla 1.21.10 '
            'FrogspawnBlock: fluid below WATER + above EMPTY '
            '(javap-verified on the client-named jar, uses NO supports_* '
            'tag). The port reads the absent #supports_frogspawn tag in '
            'both survival implementations: block_survival.rs:609 (block) '
            '+ :619 (fluid) and survive.rs:477-478. The tag is ABSENT '
            'from 1.21.10 data (the jar ships ZERO supports_* block tags), '
            'so frogspawn can never survive on valid ground. No '
            'synthesize_legacy_support_tags code exists in the 1.21.10 '
            'tree (the 1.21.7 tree\'s fix was never carried).'),
        'fix': None,
        'id': 'B12-G09',
        'method': 'tag census (jar vs prepared) + runtime tag-read audit + javap',
        'severity': 'major',
        'status': 'gap',
        'title': 'Frogspawn can never survive: port reads the absent #supports_frogspawn tag',
    },
]

slices = {}
for n in range(13):
    with open(f'{BASE}/batch-{n:02d}.json') as f:
        slices[n] = json.load(f)

batches = {}
tot = Counter()
# Behaviour-only restatement: remove the non-behaviour checks the
# batches counted in their as-tallied tallies (each per a 26.2 precedent,
# accepted by the coordinator): B4 37 driver-status checks (8 pass,
# 29 gap — passing-oracle + oracle-blocked driver rows), B3 3
# driver-code-defect checks (0 pass, 3 gap), B9 4 oracle/harness checks
# (1 pass, 3 gap — H01 is FIXED for 1.21.10, so it counts as a pass).
# Total removed: 44 checks (9 pass, 35 gap).
NON_BEHAVIOUR = {4: (8, 29), 3: (0, 3), 9: (1, 3)}
behav_tot = Counter()
raw_items = 0
for n, d in slices.items():
    c = dict(d['checks'])
    if n in TALLY_CORRECTIONS:
        c.update(TALLY_CORRECTIONS[n])
    batches[str(n)] = {
        'name': d.get('name', f'Batch {n}'),
        'date': d.get('date', '2026-11-25'),
        'checks': c,
        'status': 'complete',
    }
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
        status = it.get('status', 'gap')
        coord_note = None
        if it['id'] in CORRECTIONS:
            status, title, coord_note = CORRECTIONS[it['id']]
            ev = ev + '\n\n[Coordinator correction: ' + coord_note + ']'
        notes = []
        if it['id'] in FOLD_NOTES:
            notes.append(FOLD_NOTES[it['id']])
        elif it['id'] in absorbed:
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
        merged_item = {
            'evidence': ev,
            'fix': None,
            'id': it['id'],
            'method': it.get('method', ''),
            'severity': it.get('severity', 'minor'),
            'status': status,
            'title': title,
        }
        if coord_note:
            merged_item['coordinator_note'] = coord_note
        items.append(merged_item)

items.extend(MINTED)

sev = Counter(i['severity'] for i in items)
st = Counter(i['status'] for i in items)
live = [i for i in items if i['status'] in ('gap', 'oracle-defect', 'needs-harness')]
live_major = [i for i in live if i['severity'] == 'major']

merged = {
    'version': '1.21.10',
    'generated_from': ('docs/PARITY-SWEEP.md (branch parity-sweep); '
                       'per-batch slices in pins/1.21.10/sweep/batch-NN.json'),
    'note': (
        'Sweep ledger for 1.21.10, merged 2026-11-25 from the 13 per-batch slices. '
        'Same shape as the 26.1.2 merged ledger (pins/26.1.2/sweep-items.json): '
        'one item per defect found, with evidence, method, severity, status. '
        'Fixes are separate work, never on this branch; every fix field is null. '
        'ID convention: all ids are batch-prefixed (B<n>-Gxx behaviour/driver, '
        'B<n>-Oxx oracle, B<n>-Hxx needs-harness, O1-O13 Batch 0 repair groups). '
        'PROVENANCE: 1.21.10 is template-derived from the 1.21.11 template '
        '(first adjacent delta port in E6, cross-protocol 774->773; 605 files '
        'differ from the template; behaviour port 719/719 merged fcb0e7e3, '
        'crate-tests 45376855229 fully green). Holder shape is 1.21.x (no '
        'EntityTypes/BlockEntityTypes in either jar). '
        'TALLY CONVENTION: the "totals" block gives both the as-tallied total '
        '(every batch slice tally summed as delivered, with the B5 correction '
        'below) and the behaviour-only total. Three batches count non-behaviour '
        'checks in their as-tallied tallies, each per a 26.2 precedent the '
        'coordinator accepted: Batch 4 counts 37 driver-status checks (8 pass, '
        '29 gap), Batch 3 counts 3 driver-code-defect checks (0 pass, 3 gap), '
        'Batch 9 counts 3 oracle items + 1 harness blocker as checks (1 pass, '
        '3 gap — H01 is fixed for 1.21.10, so it counts as a pass). The '
        'behaviour-only total removes all 44 (9 pass, 35 gap). No other batch '
        'counts oracle-status checks. '
        'COORDINATOR CORRECTIONS (verified against the tree, the jars, and the '
        'Batch 0 measured table): B5 tally 101/27 -> 102/26 (the port honours '
        'doFireTick via gamerules::fire_radius; B1-G11 stays FIXED). B7-O11 '
        'fixed -> oracle-defect (KnockbackClient fails all three classpaths, '
        '7 no-override errors; the 26.1.2 cause is gone but a new cause '
        'replaces it). B5-O03 oracle-defect -> fixed (VanillaPickBlockDataOracle '
        'compiles; TypedEntityData HAS both loadInto overloads). B12-G09 '
        '(frogspawn) minted at merge from B12-G03\'s evidence. '
        'STATUS MEANINGS: "gap" = demonstrated defect in 1.21.10. '
        '"oracle-defect" = an oracle that fails javac against the 1.21.10 jars '
        '(Batch 0 measured table). "fixed" means "1.21.7 defect withdrawn for '
        '1.21.10 — verified absent", NOT a fix applied during this sweep. '
        '"closed" means "1.21.7 defect re-checked and closed for 1.21.10". '
        '"accepted-gap" = known, accepted. BATCH 0 vs PER-BATCH: Batch 0 items '
        'O1-O13 are repair-group items (one per failure class); per-batch '
        'O-items are the same oracles recorded per driver family. Both are '
        'kept; the overlap is mapped in "dedup".batch0_overlap — it is not '
        'double-counting, it is two views of the same 287 failing files. '
        'ORACLE COVERAGE: the per-batch O-items name 168 distinct files, 152 '
        'genuinely failing (16 named files pass — cross-references or passing '
        'comparators, including VanillaPickBlockDataOracle); 135 failing files '
        'have no per-batch record; the Batch 0 groups name 114 files (111 '
        'failing); 82 failing files are named by neither (the long tail, '
        'covered by O7\'s count). B3-O04 EntityTranscript/RideTranscript are '
        'driver/transcript files, not oracles. MEASUREMENT: command_parse ran '
        'end-to-end for 1.21.10 (Batch 9 §2 — H01 fixed, the probe compiles '
        'AND links; corpus regenerated from 1.21.10\'s own tree export, '
        '32,365 commands, 97,095 comparisons, 90,457 match); it is the only '
        'end-to-end driver run in this sweep. The loot_fill example WAS run '
        'for 1.21.10 (Batch 12 demonstrated G02 live). Batch 9\'s 18 other '
        'e2e drivers are unrun (command_exec fixture blocker + '
        'VanillaCommandExecOracle 45 errors). LANG: prepared en_us.json IS '
        'present (horizon-client/1.21.10/assets/minecraft/lang/en_us.json); '
        'only sounds.json is genuinely absent (B10-G02). B9-O04 (prepared data '
        'lacks the lang file) is a snapshot gap in B9\'s own run copy, not the '
        'prepared tree. '
        'NEW vs 1.21.7/26.1.2: the six supports_* survival blocks (B5-G37-'
        'G42) + frogspawn (B12-G09); B9-O05 (VanillaCommandExecOracle, 45 '
        'errors); O10 (ServerLevel ctor, 86 files — 1.21.10\'s ctor is 11-arg '
        'with RandomSequences; the 1.21.7 12-arg fix does NOT carry); B6-G18 '
        '(ore veins never generate); B5-G36 (client bonemeal inverted); '
        'B4-O27 (VanillaTradeComponentsOracle); B3-G48 (zombie horses burn), '
        'B3-G49 (LAN idle timer); B1-G09 (fire burnout); B12-G04 (Assets::load '
        'hard-requires three 26.x-only motion tags — the client cannot start); '
        'B11-G10 (vanilla-written worlds load as void — inherited from the '
        '1.21.11 template, statically derived). Two 1.21.7 items land closed '
        'as NEVER EXISTED (B5-G17 chorus ceiling — getMaxY() is 319 in BOTH '
        'jars; B5-G18 bell isProperHit), and B2-G11/G12 + B10-G04 are '
        'corrected as NEVER EXISTED against 1.21.10\'s own evidence — the '
        'B10-G04 correction extends to the 1.21.7 ledger itself (the 1.21.7 '
        'tree also has all five geometry functions with live call sites).'
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
            'O1 (LocalPlayer ctor, 20 oracles)': 'Per-batch records: B10-O04 '
                '(3 sound oracles), B3-O02, B7-O04, B7-O21, B7-O22, B7-O28, '
                'B8-O01, B10-O03, B10-O05 (1 each). 9 oracles (AutoJump, '
                'Camera, GameLoop, LocalInput, PlayerSync, RemoteAttack, '
                'SheepDye, SprintInput, TeamRules) appear only here.',
            'O2 (jsr305 client-libraries crash)': 'Latent driver defect; no '
                'per-batch item (test_model_baking_parity.py client_libraries '
                'omits jsr305).',
            'O3 (fuelValues + missing overrides, 6 oracles)': 'Per-batch '
                'records: B3-O01 (3 fuelValues oracles), B3-O11 (2), '
                'B10-O03 (2), B4-O15 (BlockEntityAnim), B6-O05, B4-O16 (1 '
                'each). None appear only here.',
            'O4 (densityfunction, 17 oracles)': 'Per-batch records: B6-O01 '
                '(10), B6-O02 (5), B6-O07 (2). 1 oracle (Noise) appears only '
                'here.',
            'O5 (EntityTypes holder move, 32 oracles)': 'Per-batch records: '
                'B10-O03 (2), B6-O06 (2), B3-O10 (2), B3-O09 (2), and 17 '
                'items with 1 each (B7-O09, B3-O02, B7-O08, B3-O05, B4-O25, '
                'B7-O04, B4-O15, B4-O22, B4-O23, B6-O08, B4-O08, B4-O09, '
                'B4-O10, B10-O05, B4-O06, B5-O06, B7-O01). 14 oracles '
                '(BreezeHook, CommandExec, KnockbackClient, MobAi, '
                'MobFallReset, RegistryFallRay, RemoteAttack, SceneMotion, '
                'ShulkerBullet, SourceSilence, SprintInput, SquidClient, '
                'WetFire, WorldRay) appear only here.',
            'O6 (other 26.3/1.21.11-only packages)': 'O6 names only 7 '
                'oracles in its evidence (it is a package-count group: '
                'gamerules 24, entity sub-packages 32, world.attribute 28, '
                'gizmos 4, permissions 9); 2 have per-batch records (B3-O09, '
                'B10-O05). The package groups are covered per-batch by group '
                'items: B3-O09 (gamerules package, 6 oracles), B3-O10 '
                '(entity sub-packages, 6 oracles), B6-O02 (material/rule, 4 '
                'oracles), B10-O05 (renderpearl, 8 oracles). 5 oracles '
                '(CommandExec, DataFix, EntityComponents, FrameLimiter, '
                'MobAi) appear only here.',
            'O7 (287 fail / 314 drivers blocked)': 'Headline roll-up; every '
                'per-batch oracle item is a subset of these 287. 82 failing '
                'files are named by neither the B0 groups nor the per-batch '
                'items (the long tail).',
            'O8 (runtime layer unmeasured)': 'No per-batch item. Partially '
                'measured now: command_parse ran end-to-end (Batch 9 §2) and '
                'loot_fill ran (Batch 12); the remaining runtime layer (other '
                'drivers) is still unmeasured.',
            'O9 (re-ported 26.3-form oracle sources, 17 files)': 'Per-batch '
                'records: B6-O03 (2), B6-O05 (2), B4-O04, B4-O05, B5-O04, '
                'B10-O03, B6-O06 (1 each). 9 oracles (BasicFeature, '
                'BlockTransformer, FeatureGraph, FeatureRegion, FeatureSort, '
                'FillLayer, ScatteredOre, SimpleBlock, SteppedInterpolation) '
                'appear only here.',
            'O10 (ServerLevel ctor, 86 oracles)': 'NEW for 1.21.10: 1.21.10\'s '
                'ServerLevel ctor is 11-arg with RandomSequences; the 1.21.7 '
                '12-arg fix does NOT carry. Per-batch records: 5 items '
                '(B2-O03, B4-O01, B2-O05, B7-O02, B7-O30, 1 oracle each). 20 '
                'oracles appear only here (ActiveEffects, '
                'AttributeMapQueries, BlockEventReadiness, BlockLanding, '
                'CombatRules, ConsumeEffect, DamageReduction, EdgeBackoff, '
                'FallDamage, FoodProperties, GroundState, '
                'LiveAttributeUpdates, LivingDamage, MovementEmission, '
                'MovementSupport, ServerPlayerDamage, StoredConsume, '
                'StoredEffectNotifications, Swimming, WaterSplash).',
            'O11 (MobAi driver gates)': 'No per-batch item; driver-gating '
                'finding (20 scenarios ungated: spears + age lock).',
            'O12 (ComponentPatch runtime crash)': 'No per-batch item; the '
                'oracle COMPILES (B0-O12 is a runtime crash, not javac) — '
                'the only runtime-measured oracle failure in this sweep '
                'besides the command_parse run.',
            'O13 (withdrawn format-change repairs)': 'CLOSED: no such repairs '
                'in the 1.21.10 tree.',
        },
    },
    'items': items,
}

out = '/home/hatch/workspace/mcir-work/sweep-1.21.10/pins/1.21.10/sweep-items.json'
with open(out, 'w') as f:
    json.dump(merged, f, indent=1, sort_keys=True)
print('wrote', out)
print('checks:', dict(tot))
print('behaviour-only:', dict(behav_tot))
print('items raw:', raw_items, '| unique:', len(items), '| live:', len(live),
      '| live major:', len(live_major))
print('by status:', dict(st))
print('by severity:', dict(sev))
print('live majors:', sorted(i['id'] for i in live_major))
