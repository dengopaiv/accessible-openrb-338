---
id: TASK-272
title: >-
  The original ReBirth hangs ("Not Responding") when switching to the recrypted
  cold-rolled.rbm
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-21T12:56:00Z'
labels:
  - rbm
  - re
  - testmachine
milestone: m-5
dependencies: []
references:
  - docs/rbm.md
  - docs/test_machine.md
  - crates/rbm
  - tools/rbm.py
priority: medium
type: bug
ordinal: 258000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Reported by the user on 2026-11-21. On the Windows test machine, the original ReBirth 2.01 hangs in "Not Responding" when the user switches to the mod `cold-rolled.rbm` from their collection `~/rb-mods/mods`. Those are mods recrypted by `orb mod recrypt` (docs/rbm.md section 12).

Recrypted mods are meant to stay loadable by the original:
- docs/rbm.md 12.3 records the four shipped mods, recrypted, opening and exporting correctly in the original.
- crates/rbm/tests check that every file and slot sample of all third-party recrypted mods equals the original's.
So a hang means either orb's recrypted form of this mod is something the original can't handle, or the original has a defect this mod triggers.

**What to find out.**
1. **Reproduce on the test machine** with the original and the recrypted cold-rolled.rbm (docs/test_machine.md; take the machine's permits and touch only your own files and processes, since the machine is shared). Is the hang real (CPU-bound loop, or blocked), or a very slow load? Time it.
2. **Compare with the unrecrypted original of cold-rolled.rbm** if it can be found:
   - the collection's sources are credited in ~/rb-mods/README.md (audiovst.mnx2010.nl/prop and github.com/peffre/ReBirth-RB-338);
   - the user's earlier collection notes are in docs/rbm.md section 11.
   Does the original form hang too? If it does, it's a defect of the original: document it in docs/original_bugs.md and in docs/rbm.md, and note whether orb handles that mod.
3. **If only the recrypted form hangs:** find where in Rebirth.exe / ReBirth.dll it loops or blocks, and why. Use a debugger on the test machine, or IDA on copies of the user's databases (/Users/pingw33n/dev/openrb-338/ida/*.i64, never saved). Look at:
   - the mod reader's chunk walk (docs/rbm.md 12.3 notes that chunk count affects load time);
   - the key window;
   - this mod's size, chunk structure and sample count against the other 319.
4. **Survey the collection:** find whether other recrypted mods in ~/rb-mods/mods hang or load slowly in the original. A script on the test machine timing each mod switch would do; batch it if it's long.
5. **If orb's recrypt output is at fault:** fix `orb mod recrypt` (crates/rbm). Re-recrypting must stay byte-exact for the mods that work (pinned hashes in docs/test_machine.md 7 and tests/refs/thirdparty_mods.json), or the change must be justified and the manifests regenerated. Then say what the user must do with their collection (re-recrypt, and republish the rb-mods repository) as a follow-up task for the user.

Every fact about the original goes into docs/ with addresses, [C] or [G]. No test or tool may need the original's files: use ORB_CD_ISO, and skip without it.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The hang is reproduced on the test machine and characterized (loop or block, or slow load, with timing), with the evidence in the notes
- [ ] #2 The original (unrecrypted) form of cold-rolled.rbm is tested in the original if it can be found, or the notes say why not
- [ ] #3 The cause is found with addresses and documented in docs/rbm.md (and docs/original_bugs.md if it is the original's defect)
- [ ] #4 The collection is surveyed for other mods that hang or load slowly in the original; the result is in the notes
- [ ] #5 If orb's recrypt is at fault it is fixed with tests and the working mods' outputs stay byte-exact (or the change is justified and the manifests regenerated), and a follow-up task tells the user what to do with the collection
<!-- AC:END -->
