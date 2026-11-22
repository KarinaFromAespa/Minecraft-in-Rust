# Sweep evidence

Working files behind the batch verdicts in `docs/sweep/batch-N-*.md` and
the ledger in `pins/1.21.7/sweep-items.json`. These are the audit trail:
without them the verdicts they support cannot be re-checked or
regenerated. Committed 2026-11-22 on the owner's word.

- `batch-4/` — jar evidence for the block-entities/containers batch:
  `NOTES.md` plus `javap` dumps for anvil, cartography table,
  enchantment codec, merchant and trial vault classes.
- `batch-6/` — helper scripts for the worldgen/features batch:
  `coldump.py` (column dumps) and `nbtmini.py` (minimal NBT reader).
- `batch-9/` — classification tooling for the commands batch:
  `classify_cmdparse.py`, `classify_cmdparse2.py` and the resulting
  `cmdparse-classification.txt` (the 86,241 error-text mismatches are
  a probe defect, not game defects — see batch-9 report).

The batch reports that cite these files may still name their old
workdir paths under `~/workspace/mcir-work/sweep-batch-N-deliverables/`;
the copies here are the committed originals.
