# Port notes — 26.3 (base)

26.3 is the imported upstream base, not a port: the tree is the upstream
snapshot at 962f5949ac, unmodified except for the programme's reframed front
docs and the research-lock maintenance below. Its CI profile defines the
Phase 0 gate baseline ("baseline" = the 26.3 control profile; see README).

## Control run 14 (43131496845) — no baseline produced

Dispatched 2026-11-12 01:01Z on port-verify v6 (main@6567681). The job
reported `completed/success`, but the artifacts show it measured nothing:
profile 0 pass / 0 oracle / 17 fail, prepare failed. Two independent
causes, both found by reading the logs rather than the colour:

1. **Mojang republished the 26.3 version JSON.** Lab-prepare refused the
   staged launcher directory: the version file's sha1,
   `9f67e10ed013deeaad20ac3c800652765d493408`, was not in the lock's
   accepted list. Verified independently against Mojang's own version
   manifest (the manifest lists that exact sha1 for 26.3; jars and
   libraries are unchanged, only the asset index differs — the sixth
   republication of this file, the lock already listed five). The new
   asset index (id 34, sha1 `e99f9973047785b5a9d52eaf72020b97545053c9`)
   was also unlisted. Both sha1s were added to `research-lock.json`
   (data only, no code change). Pipeline prepare itself succeeded
   (11,383 classes indexed); only the lab/harness prepare validates
   the version file against the lock.

2. **v6's gates did not gate.** The runner's shell is `bash -e` WITHOUT
   pipefail, so every `cmd 2>&1 | tee log` step returned tee's status:
   the compile gate failed (alsa-sys build script — pkg-config could
   not find libasound, because the apt step ran AFTER the gate) and
   lab-prepare failed, and both steps still showed success. The
   profile emitter also counted 0 compile errors for a failed build,
   because cargo's coloured `error:` lines do not start with "error".

Fixed in port-verify v6.1: `set -o pipefail` in every step that pipes to
tee; apt install moved before the compile gate; cargo runs with
`--color=never`; the emitter strips ANSI escapes before counting error
lines (verified against run 14's logs: 0 -> 1 error on both check logs).

Control re-run dispatched after these fixes; the baseline is pending
until it produces a profile from a green prepare.
