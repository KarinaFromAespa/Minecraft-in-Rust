# Playtest — 26.2

release: 26.2
commit: 3f3c7d287e9f07ef82e854eef16d3dbc5fff35be
tester: V-Cool

- [x] world loads from a level.dat saved by 26.2
- [x] movement, swimming, elytra
- [x] combat, knockback and shield
- [ ] containers, crafting, anvil
- [ ] a redstone sample machine ticks like vanilla
- [ ] worldgen matches a vanilla seed at spawn
- [ ] vanilla 26.2 client joins the LAN world
- [x] save round-trip: play, save, reopen

evidence: playtest-run.log

PARTIAL RECORD, 2026-11-18: 4 of 8 boxes ticked. D6 does NOT pass on
this sheet — the gate needs every listed box ticked, and the LAN join
is a D6 must-hold that was not tested. The tester deferred the
remaining boxes to a future session (redstone sample, worldgen against
a vanilla seed, LAN join); the containers box is part-done (chest and
crafting table verified, anvil untested).

Session notes: three playtest-found bugs were fixed and verified
in-game in this session — the /swing echo (d987f100), the World Options
menu split and the loot-drops kind dispatch (a367b809, 3f3c7d28). Two
reported issues turned out not to be bugs: firework rockets work when
used per vanilla rules (block click to launch, boost only while
gliding), and creative flight underwater is vanilla-correct.

Item 653 (block-entity sherds) is still unmerged on be-sherds/26.2, so
a completed D6 will be recorded against the post-merge HEAD.

The stamped commit is the tested build. Recording this sheet is itself
a commit, so HEAD moves past the stamp — the same quirk as the profile
stamp (patch 0006). A completed D6 will need the same treatment.
