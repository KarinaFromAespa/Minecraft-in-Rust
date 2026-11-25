#!/usr/bin/env python3
"""Merge the 1.21.11 sweep per-batch slices into pins/1.21.11/sweep-items.json.

Mirrors the 1.21.10 merged ledger shape (version, generated_from, note,
batches, items, totals, dedup). TALLY: the "totals" block gives both the
as-tallied total (every batch's slice tally summed as delivered) and the
behaviour-only total. NON_BEHAVIOUR (below) is filled at merge time from
the actual non-behaviour checks the batches counted in their as-tallied
tallies (26.2 precedent: Batch 4 driver-status, Batch 3 driver-code-defect,
Batch 9 oracle/harness checks).

Whole-item duplicates are REMOVED from the merged items list (kept in their
slice files); the canonical item's evidence gains a [Merge note]. Per-oracle
member overlaps inside group items are annotated on both sides.

COORDINATOR VERIFICATION (verified by the coordinator against the tree,
the jars, the prepared registries, and the Batch 0 measured table — see
sweep-1.21.11-batches/merge-notes.md):
- All 13 batches landed; every slice verified (report tail read, no
  truncation markers, slice JSON valid, no 'fix' key, no coordinator_note,
  no duplicate ids). B9's corpus md5 verified against the delivered file.
- Oracle reconciliation: 106 singleton oracle items checked against the
  landed Batch 0 table — 0 real contradictions. The 3 apparent flags
  (B7-O09 ItemModel, B8-O04 EmitterParticles, B8-O05 EntityParticles)
  claim FIXED while the table says fail/pass/pass: all three are client
  oracles whose evidence explicitly says they pass on client + both; the
  server-classpath failure is the expected artifact for client-only
  classes, not a contradiction.
- B9-G01's withdrawal verified: the prepared registries.json shows 59
  minecraft:game_rule entries with snake_case ids (minecraft:advance_time,
  minecraft:block_drops, ...) — vanilla 1.21.11 uses snake_case, the port's
  table matches exactly.
- B9-O01/O02/O03 error counts (3/6/17) confirmed exact against the B0
  table; B9-O05 (CommandExec) confirmed pass on all three classpaths.
- NO coordinator corrections at merge: every landing correction was
  applied to the slices themselves (B5-O02 flipped to oracle-defect at
  landing; B7-O11 corrected by the worker itself; B3's 142 claim corrected
  to B0's 127 at landing). NO tally corrections. NO items minted at merge
  (B12-G09 frogspawn was already minted in the B12 slice).

WHOLE_FOLDS were verified fresh against the 1.21.11 slices: every
participant exists, every pair is live, and the duplicate/canonical
evidence was compared (splash-water 3-way, mud, spawner, dripstone tag
half, snow-layer tags, loot condition() defect, cauldron 3-way, dispenser
3-way, lava-cauldron 3-way, render/UI singletons -> B10 families all
confirmed). Differences from the 1.21.10 folds, each decided from the
1.21.11 evidence:
- NEW: B4-G26 -> B12-G01 (the items.rs condition() defect; B4's evidence
  adds the block-table blast radius 856/1,083 + the test_crafter_drop_name
  flip, B12's adds the 1,190/1,324 table census + residual 102).
- NEW: B11-G14 -> B5-G08 (same snow-layer tag defect; B11-G14 adds the
  golem roll-order half, which is FIXED).
- DROPPED: B12-G03 -> B5-G37 (no B12-G03 exists in 1.21.11 — the census
  item was never minted; frogspawn is already B12-G09 in the slice).
- DROPPED: B3-O08 -> B1-O06 (B3-O08 is CLOSED in 1.21.11, not a live
  duplicate — the void error is gone; B1-O06 stands as the live record).
- DROPPED: B7-O09 -> B10-O03 (B7-O09 is FIXED in 1.21.11, and B10-O03's
  evidence already lists VanillaItemModelOracle as passing in the family).
- KEPT SEPARATE per the 1.21.10 precedent: B11-G07 (recipe-book fuel list
  EMPTY) vs B4-G01 (furnaces never light) — different behaviours, same
  root cause; B12-G04 (Assets::load motion tags) vs B3-G22
  (MotionResources::load) — different load paths.
"""
import json
from collections import Counter

BASE = '/home/hatch/workspace/mcir-work/sweep-1.21.11/pins/1.21.11/sweep'

# Whole-item folds: duplicate id -> canonical id. 23 folds.
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
    # Snow-layer survival tags: B11-G14 is the same defect as B5-G08
    # (same block, same method, same tag names); the golem half is FIXED.
    'B11-G14': 'B5-G08',
    # Loot condition() defect: B4-G26 folds into B12-G01 (the fuller
    # measurement: 1,190/1,324 tables, residual 102; B4's evidence adds the
    # block-table blast radius + the test_crafter_drop_name flip).
    'B4-G26': 'B12-G01',
    # Cauldron 3-way, canonical B1-O02.
    'B5-O01': 'B1-O02',
    'B11-O01': 'B1-O02',
    # FallingBlock.
    'B5-O05': 'B1-O03',
    # LavaCauldronItem 3-way, canonical B1-O07.
    'B7-O07': 'B1-O07',
    'B11-O02': 'B1-O07',
    # Dispenser 3-way, canonical B4-O06 (26.2 precedent).
    'B5-O06': 'B4-O06',
    'B7-O01': 'B4-O06',
    # BlockUpdate, 38 errors.
    'B7-O02': 'B2-O01',
    # FaceSupport.
    'B5-O09': 'B2-O06',
    # VehicleContainer.
    'B7-O10': 'B4-O21',
    # EnchantDurability cascade (both cascade on BlockUpdate).
    'B7-O29': 'B4-O19',
    # Render / UI family singletons -> B10 group items (26.2 precedents).
    # B7-O09 (ItemModel) is NOT folded: it is FIXED in 1.21.11 and B10-O03
    # already lists it as passing in the family.
    'B5-O07': 'B10-O03',   # SectionMesh singleton -> render family
    'B5-O08': 'B10-O03',   # BreakingModel singleton -> render family
    'B4-O16': 'B10-O03',   # BlockEntityRender (26.2 precedent exactly)
    'B4-O18': 'B10-O05',   # RecipeBookScreen (26.2 precedent exactly)
    'B4-O08': 'B10-O05',   # MenuOracle singleton -> UI family
}

# Custom merge notes for canonicals whose fold needs more than the default
# "absorbs duplicate" text.
FOLD_NOTES = {
    'B12-G01': ('Absorbs B4-G26 (same items.rs condition() defect: the kind '
                'is read from o.get("type") only, while 1.21.11 uses the '
                '"condition" key). B4-G26\'s evidence adds the block-table '
                'blast radius (856/1,083 block tables drop nothing) and the '
                'test_crafter_drop_name flip; B12-G01\'s evidence adds the '
                '1,190/1,324 table census and the post-fix residual of '
                'exactly 102.'),
    'B5-G08': ('Absorbs B11-G14 (same snow-layer tag defect: '
               'SnowLayerBlock.canSurvive reads the 26.x tag names; '
               '1.21.11 uses #SNOW_LAYER_CANNOT_SURVIVE_ON / '
               '#SNOW_LAYER_CAN_SURVIVE_ON). B11-G14\'s evidence adds the '
               'golem roll-order half, which is FIXED.'),
    'B1-G03': ('Absorbs B7-G10 and B11-G05 (same splash-water defect: '
               'vanilla gates on PotionContents.is(Potions.WATER); the port '
               'gates all four effects on potion tags that do not exist in '
               '1.21.11).'),
}

# Per-oracle member folds: (source group item, oracle) -> canonical item.
# Cascade/dependency mentions are NOT members (e.g. BlockUpdate named in
# B4-O03/B4-O19/B7-O29/B7-O30/B3-O07 as the cascade source; FaceSupport
# named in B2-O07/B3-O07 as the cascade source; RegistryFluid named in
# B1-O04 as a shared-symbol cross-reference) — those get no fold.
# Differences from the 1.21.10 list, each verified against the 1.21.11
# evidence: the four B3-O10 triples are dropped (B3-O10 is CLOSED in
# 1.21.11 and its evidence does not name those oracles); the B6-O02
# MaterialSurface triple is dropped (B6-O02 does not name it); the three
# NaturalSpawn triples from B3-O09/B6-O06/B6-O08 are dropped (those groups
# do not name it in 1.21.11) and replaced by the groups that DO name it
# (B3-O04, B3-O06, B6-O09); the two SpawnRules triples are dropped
# (B6-O06/B6-O08 do not name it; B3-O10 is closed; SpawnRules is recorded
# only as a member of B3-O07 (cascade) and B6-O09, with no singleton).
# NEW: PlayerEquipment (B3-O05 -> B7-O36) and the three NaturalSpawn folds.
MEMBER_FOLDS = [
    ('B3-O11', 'VanillaActiveEffectTickOracle', 'B3-O01'),
    ('B3-O11', 'VanillaAttributeUpdatesOracle', 'B3-O01'),
    ('B10-O03', 'VanillaBlockEntityAnimOracle', 'B4-O15'),
    ('B10-O05', 'VanillaContainerScreenOracle', 'B4-O17'),
    ('B3-O02', 'VanillaEntitySoundPacketOracle', 'B10-O04'),
    ('B10-O05', 'VanillaRecipeBookOracle', 'B4-O09'),
    ('B10-O05', 'VanillaPlaceRecipeOracle', 'B4-O10'),
    ('B10-O05', 'VanillaSimpleContainerOracle', 'B4-O11'),
    ('B3-O05', 'VanillaTradeComponentsOracle', 'B4-O27'),
    ('B3-O05', 'VanillaPlayerEquipmentOracle', 'B7-O36'),
    ('B3-O04', 'VanillaNaturalSpawnOracle', 'B4-O23'),
    ('B3-O06', 'VanillaNaturalSpawnOracle', 'B4-O23'),
    ('B6-O09', 'VanillaNaturalSpawnOracle', 'B4-O23'),
]

# Coordinator corrections: none at merge. Every landing correction was
# applied to the slices themselves; the oracle reconciliation found 0
# real contradictions.
CORRECTIONS = {}

# Tally corrections: none. B5's tally is as delivered (fire tick passes,
# counted as a pass; no doFireTick correction is needed for 1.21.11).
TALLY_CORRECTIONS = {}

# Items minted at merge: none. B12-G09 (frogspawn) was already minted in
# the B12 slice.
MINTED = []

slices = {}
for n in range(13):
    with open(f'{BASE}/batch-{n:02d}.json') as f:
        slices[n] = json.load(f)

batches = {}
tot = Counter()
# Behaviour-only restatement: remove the non-behaviour checks the
# batches counted in their as-tallied tallies (each per a 26.2 precedent,
# accepted by the coordinator): B4 37 driver-status checks (9 pass,
# 28 gap — passing-oracle + oracle-blocked driver rows), B3 3
# driver-code-defect checks (1 pass, 2 gap), B9 4 oracle/harness checks
# (1 pass, 3 gap — H01 is FIXED for 1.21.11, so it counts as a pass).
# Total removed: 44 checks (11 pass, 33 gap).
NON_BEHAVIOUR = {4: (9, 28), 3: (1, 2), 9: (1, 3)}
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
    'version': '1.21.11',
    'generated_from': ('docs/PARITY-SWEEP.md (branch parity-sweep); '
                       'per-batch slices in pins/1.21.11/sweep/batch-NN.json'),
    'note': (
        'Sweep ledger for 1.21.11, merged 2026-11-25 from the 13 per-batch slices. '
        'Same shape as the 1.21.10 merged ledger (pins/1.21.10/sweep-items.json): '
        'one item per defect found, with evidence, method, severity, status. '
        'Fixes are separate work, never on this branch; every fix field is null. '
        'ID convention: all ids are batch-prefixed (B<n>-Gxx behaviour/driver, '
        'B<n>-Oxx oracle, B<n>-Hxx needs-harness, O1-O13 Batch 0 repair groups). '
        'PROVENANCE: 1.21.11 is the E6 TEMPLATE ITSELF — the template whose '
        'defects the 1.21.10 sweep found inherited. Behaviour port 815/815 '
        'merged 9de51367; CI proof run 44989513836 (profile 5 pass / 18 fail '
        'of 23; D1 fail on 21 dead tags, D2 pass, D3 pass 502/502). Holder '
        'shape is 1.21.x (no EntityTypes/BlockEntityTypes in either jar). '
        'DataVersion 4671, protocol 774, Java 21, data pack 94.1, resource '
        'pack 75.0. '
        'TALLY CONVENTION: the "totals" block gives both the as-tallied total '
        '(every batch slice tally summed as delivered) and the behaviour-only '
        'total. Three batches count non-behaviour checks in their as-tallied '
        'tallies, each per a 26.2 precedent the coordinator accepted: Batch 4 '
        'counts 37 driver-status checks (9 pass, 28 gap), Batch 3 counts 3 '
        'driver-code-defect checks (1 pass, 2 gap), Batch 9 counts 3 oracle '
        'items + 1 harness blocker as checks (1 pass, 3 gap — H01 is fixed '
        'for 1.21.11, so it counts as a pass). The behaviour-only total '
        'removes all 44 (11 pass, 33 gap). No other batch counts '
        'oracle-status checks. B12 counting note: 11 supports_* + 3 '
        'motion-tag reads were verified as re-check evidence but NOT counted '
        'as checks per the brief (1.21.10 counting would add 14 gap checks, '
        'total 8,304 — flagged, not applied; 8,290 stands as delivered). '
        'COORDINATOR VERIFICATION: no corrections at merge — every landing '
        'correction was applied to the slices themselves (B5-O02 flipped to '
        'oracle-defect at landing; B7-O11 corrected by the worker itself; '
        'B3\'s 142 claim corrected to B0\'s 127 at landing). Oracle '
        'reconciliation: 106 singleton oracle items checked against the '
        'landed Batch 0 table, 0 real contradictions (the 3 apparent flags '
        'are client oracles failing only on the server classpath). No tally '
        'corrections. No items minted at merge (B12-G09 frogspawn was already '
        'minted in the B12 slice). '
        'STATUS MEANINGS: "gap" = demonstrated defect in 1.21.11. '
        '"oracle-defect" = an oracle that fails javac against the 1.21.11 jars '
        '(Batch 0 measured table). "fixed" means "1.21.7/1.21.10 defect '
        'withdrawn for 1.21.11 — verified absent", NOT a fix applied during '
        'this sweep. "closed" means "1.21.7/1.21.10 defect re-checked and '
        'closed for 1.21.11". "accepted-gap" = known, accepted. STATUS '
        'CONVENTION NOTE: all 1.21.11 batches follow common.md ("fixed" = '
        're-check verified absent in the 1.21.11 tree; "closed" = premise '
        'fails on 1.21.11 evidence) — the REVERSE of the 1.21.10 slice\'s '
        'usage. No normalization was needed within this run\'s slices. '
        'BATCH 0 vs PER-BATCH: Batch 0 items O1-O13 are repair-group items '
        '(one per failure class); per-batch O-items are the same oracles '
        'recorded per driver family. Both are kept; the overlap is mapped '
        'in "dedup".batch0_overlap — it is not double-counting, it is two '
        'views of the same 264 failing files. '
        'ORACLE COVERAGE: the per-batch O-items name 170 distinct files '
        '(168 in the Batch 0 table, 152 genuinely failing, 16 named files '
        'pass — cross-references or passing comparators, including '
        'VanillaCommandExecOracle; ProtocolClient and SquidClient are driver '
        'clients named in B3-O04, not oracles); 137 failing files have no '
        'per-batch record; the Batch 0 groups name 118 files (114 failing); '
        '72 failing files are named by neither (the long tail, covered by '
        'O7\'s count). B3-O04 EntityTranscript/RideTranscript are '
        'driver/transcript files, not oracles. MEASUREMENT: command_parse '
        'ran end-to-end for 1.21.11 (Batch 9 §2 — H01 fixed, the probe '
        'compiles AND links; corpus regenerated from 1.21.11\'s own tree '
        'export, 34,492 commands, md5 9e9597f33c9c5bc9e2112f0bda481720; perm 4: '
        '3,172 mismatches, 90.8% match; perm 2: 3,158; perm 0: 1). The '
        'loot_fill example WAS run for 1.21.11 (Batch 12 demonstrated G02 '
        'live, 7 tables x seeds 1-5). VanillaCommandExecOracle compiles '
        'clean now (B9-O05 fixed), so command_exec is one fixture away from '
        'running. Batch 9\'s 18 other e2e drivers are unrun. A VM reboot '
        '~03:10 UTC killed the first Batch 9 driver build at 232 crates; '
        'relaunched from the warm cache, finished clean in 52m00s, probe '
        'linked. LANG: prepared en_us.json IS present '
        '(horizon-client/1.21.11/assets/minecraft/lang/en_us.json); only '
        'sounds.json is genuinely absent (B10-G02). '
        'NEW vs 1.21.10: B10-G16 (the LOADING STICK EXISTS in 1.21.11 — '
        'code-verified: remote_loading() exists at game/remote.rs:10906 but '
        'none of the three pause sites is gated; deadlock not reproduced '
        'headless; propagation is a separate owner decision); B11-G10 '
        'reproduces (vanilla-written worlds load as void — the template\'s '
        'own defect, verified fresh: strict "id" only, 0 from_palette_nbt, '
        'datafix 5006 never runs at DataVersion 4671); B12-G04 (Assets::load '
        'hard-requires three 26.x-only motion tags — the client cannot '
        'start); B12-G01/B12-G02 (both loot engines dead); the B6 cascade '
        '(worldgen does not generate; 19 majors + G18 ore veins); B9-G22 '
        '(new run-found item: minecraft:container accepts entries without '
        'slot/item); B7-O31-O40 (ten new oracle defects); B6-O09 (ServerLevel '
        '11-arg ctor group, 4 oracles); O12 FIXED (ComponentPatch uses the '
        '1.21.7 repair form); O13 (template adaptations change oracle output '
        'formats — SuffocationOracle outputs motion-based rule names); '
        'B8-H04 reversed to a live gap (nautilus family arrived); B8-H05 '
        'withdrawn. WHERE VANILLA CHANGED, THE TEMPLATE IS CLEANER THAN ITS '
        'FIRST CHILD: B9-G01 withdrawn (vanilla 1.21.11\'s game_rule registry '
        'uses snake_case ids — the port matches 59=59 exactly, defaults '
        'match, 0 gamerule mismatches in 492 run cases); B9-G10 closed '
        '(vanilla uses the Time argument the tree ships); B9-G21 halved '
        '(nautilus_one_cm arrived in vanilla, only sleep_in_straw_bed is '
        'extra); B9-G19 doubly withdrawn (no /swing on either side); the '
        'permissions package arrived; the game_rule registry (59 entries) and '
        'the environment_attribute registry (45 entries) both EXIST in '
        '1.21.11 and are data-defined. The four NEVER EXISTED corrections '
        'were checked fresh against 1.21.11\'s own jar/tree and NOT re-minted '
        '(B5-G17 chorus ceiling, B5-G18 bell isProperHit, B2-G11/G12, '
        'B10-G04 BE renderers). '
        'CORRECTIONS OWED TO THE 1.21.10 LEDGER (parent actions, not this '
        'sweep): B4-G26 (the 1.21.10 report\'s test_crafter_drop_name pass '
        'was wrong — condition() reads "type" only; the 1.21.11 B4-G26/B12-G01 '
        'pair proves it) and B11-G16 NEVER EXISTED (the 1.21.10 tree reads '
        '`dyeable` too; the agent cited the :190 doc comment, not the :8-11 '
        'code).'
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
            'O1 (LocalPlayer ctor, 18 oracles)': 'Per-batch records: B3-O02 '
                '(ctor group), B10-O04 (3 sound oracles), B7-O04, B7-O21, '
                'B7-O22, B7-O28, B8-O01, B10-O03, B10-O05, B9-O02 (1 each). '
                '8 oracles (AutoJump, Camera, GameLoop, LocalInput, '
                'PlayerSync, RemoteAttack, SheepDye, SprintInput) appear '
                'only here.',
            'O2 (jsr305 client-libraries crash)': 'Latent driver defect; no '
                'per-batch item (test_model_baking_parity.py client_libraries '
                'omits jsr305; test_rust_parity.py does NOT carry the 1.21.7 '
                'server-path fix; WORSE than 1.21.10, where the server-path '
                'fix existed).',
            'O3 (fuelValues + missing overrides, 6 oracles)': 'Per-batch '
                'records: B3-O01 (3 fuelValues oracles), B3-O03 (3), B3-O11 '
                '(3), B10-O03 (2), B4-O15, B4-O16, B6-O05 (1 each). None '
                'appear only here.',
            'O4 (densityfunction, 17 oracles)': 'Per-batch records: B6-O01 '
                '(10), B6-O02 (4), B6-O04 (4), B6-O07 (3), B6-O08 (2), '
                'B6-O09 (1), plus B6-G13, B6-G17, B6-H (1 each). None appear '
                'only here.',
            'O5 (EntityTypes holder move, 30 oracles)': 'Per-batch records: '
                'B3-O04 (8), B3-O03 (4), B10-O03 (2), B6-O09 (2), and 17 '
                'items with 1 each (B7-O11, B9-O03, B3-O02, B4-O15, B7-O08, '
                'B7-O04, B9-O05, B4-O06, B5-O06, B7-O01, B6-O06, B7-O09, '
                'B4-O08, B4-O09, B4-O10, B10-O05, B3-O07, B3-O06, B4-O23, '
                'B6-O08, B4-O22, B4-O25). 12 oracles (BreezeHook, MobAi, '
                'MobFallReset, ProtocolClient, RegistryFallRay, RemoteAttack, '
                'SceneMotion, ShulkerBullet, SourceSilence, SprintInput, '
                'WetFire, WorldRay) appear only here.',
            'O6 (other 26.3/1.21.11-only packages)': 'O6 names 5 oracles in '
                'its evidence (it is a package-count group: renderpearl 8, '
                'material.rule 5, clock 13, classfile 3, model APIs, '
                'advancements); 3 have per-batch records (B9-O05, B3-O03, '
                'B10-O05). 2 oracles (DataFix, FrameLimiter) appear only '
                'here.',
            'O7 (264 fail / 289 drivers blocked)': 'Headline roll-up; every '
                'per-batch oracle item is a subset of these 264. 72 failing '
                'files are named by neither the B0 groups nor the per-batch '
                'items (the long tail).',
            'O8 (runtime layer unmeasured)': 'No per-batch item. Partially '
                'measured now: command_parse ran end-to-end (Batch 9 §2) and '
                'loot_fill ran (Batch 12); the remaining runtime layer (other '
                'drivers) is still unmeasured. O8\'s evidence names '
                'VanillaComponentPatchOracle only as the prior run\'s crash '
                'reference.',
            'O9 (re-ported 26.3-form oracle sources, 16 of 17 still fail)':
                'Per-batch records: B6-O04 (5), B6-O03 (2), B6-O05 (2), and '
                '13 items with 1 each (B4-O05, B4-O04, B10-O03, B3-O04, '
                'B3-O06, B4-O23, B5-O04, B6-O06, B6-O08, B6-O09). 2 oracles '
                '(FillLayer, SteppedInterpolation) appear only here.',
            'O10 (ServerLevel ctor, 85 oracles)': '1.21.11\'s ServerLevel '
                'ctor is 11-arg with RandomSequences; the 26.x 10-arg form '
                'does NOT carry. The evidence names 28 oracles; per-batch '
                'records: 6 items (B3-O02, B8-O07, B7-O30, B2-O05, B2-O03, '
                'B1-O09, 1 oracle each). 22 oracles appear only here '
                '(ActiveEffects, AttributeHealth, AttributeMapQueries, '
                'BlockEventReadiness, Climbing, ConsumeEffect, CrackFeedback, '
                'DamageReduction, EdgeBackoff, FallDamage, FoodProperties, '
                'GroundState, ItemUseLifecycle, LiveAttributeUpdates, '
                'LivingDamage, MovementEmission, MovementSupport, PlayerFood, '
                'ServerPlayerDamage, StoredConsume, '
                'StoredEffectNotifications, Swimming).',
            'O11 (MobAi driver gates)': 'No per-batch item; driver-gating '
                'finding (16 age-lock scenarios ungated: spears + copper '
                'golem are runnable in 1.21.11).',
            'O12 (ComponentPatch runtime crash)': 'FIXED for 1.21.11: the '
                'oracle uses the 1.21.7 repair form (Optional.empty(), no '
                'Removed reflection). No per-batch item; the oracle COMPILES '
                '(B0-O12 is a runtime crash, not javac).',
            'O13 (template adaptations change oracle output formats)':
                'NEW for 1.21.11 (1.21.10\'s O13 was closed): the tree DOES '
                'carry format-relevant oracle adaptations — '
                'VanillaSuffocationOracle outputs motion-based rule names. '
                'Per-batch record: B10-O04 (1). Suffocation appears only '
                'here.',
        },
    },
    'items': items,
}

out = '/home/hatch/workspace/mcir-work/sweep-1.21.11/pins/1.21.11/sweep-items.json'
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
