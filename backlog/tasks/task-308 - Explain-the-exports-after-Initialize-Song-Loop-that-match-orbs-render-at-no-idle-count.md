---
id: TASK-308
title: >-
  Explain the exports after Initialize Song / Loop that match orb's render at no
  idle count
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-25T14:39:00Z'
updated_date: '2026-11-25T14:47:00Z'
labels:
  - engine
  - re
milestone: m-5
dependencies: []
references:
  - docs/song_load.md
  - docs/export_state.md
  - tests/songs/edit_init/README.md
priority: low
type: spike
ordinal: 279700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** task-306 checked playback after the Edit menu's Initialize commands by exporting the loop in ReBirth 2.01 and comparing with orb's render of the song the original saved after the command (orb compare --idle-blocks N, N in steps of 16). One export matched: Initialize Loop on tests/songs/edit_init/init_loop.rbs, first session, at N 32752. Six did not match at any N searched:
- fresh sessions (one ReBirth start per export, the command about 4 s after the open, the export about 10 s after it), searched N 0..30000: Initialize Song on init_loop.rbs, Initialize Loop on init_zero.rbs (loop from bar 1), Initialize Song on init_zero.rbs;
- the first session (several songs opened one after the other in one ReBirth), searched N 0..60000: Initialize Song on init_loop.rbs, Initialize Loop and Initialize Song on init_zero.rbs.
In every failing case the first difference is at frame 66 (the first 303 sample of the loop), and in each the command changed the live 303 tune at the position (66 -> 70) while the matching one did not (its loop starts at bar 3, so the chase at position 0 changed nothing). The [G] explanation (docs/song_load.md section 14): the 303's oscillator phase runs free while idle and its increment depends on the tune, so the phase at the export depends on the idle time before and after the command, which a render of the saved file with one idle count cannot model. The saved files themselves equal orb's node for node, so the tracks are right; only this playback check is open.

**What to do:** confirm or refute the phase explanation, e.g. with an orb engine run that loads the song, renders N1 idle blocks, runs the command (Engine::init_song / init_loop), renders N2 more and then exports (orb_engine::frame::export::export without a reload), fitting (N1, N2) from the open, command and export times of fresh sessions. Or make a test song whose 303 tune is the same in the tracks and in the pattern copy, export after each command in fresh sessions and fit one N. If the explanation fails, find the real difference and fix it in orb.

**Acceptance criteria:**
- Each of the failing cases is explained with evidence (a fitted orb render equal to the export), or the difference is found and fixed in orb.
- docs/song_load.md section 14's open item is updated: [C] or the fix.

**The tools (committed in tests/songs/edit_init, see its README.md):**
- `tests/songs/edit_init/export.sh SONG loop|song OUT LOCAL_DIR`: one fresh ReBirth session on the test machine through rcs_client.py (doc-001; take all the permits of redc-win-run first: `locky acquire redc-win-run N --owner BRANCH --acquire-timeout 3600 --lease-timeout 1800`, then `locky release redc-win-run all --owner BRANCH`); prints the open and export times (the command runs about 3 s after the open, 4 s for Initialize Song with its alert) and the WAV's SHA-256. Example: `tests/songs/edit_init/export.sh init_zero.rbs song zero_song /tmp/edit_init`.
- `tests/songs/edit_init/fit.sh SAVED.rbs WAV [FROM TO STEP]`: `orb compare --idle-blocks N` from FROM to TO in steps of STEP (default 0 30000 16) against the original's save after the same command (`init_loop.after_init_song.rbs` and the like); needs `cargo build --release -p orb-cli`. Example: `tests/songs/edit_init/fit.sh init_zero.after_init_song.rbs /tmp/edit_init/zero_song.wav`.
- The reference WAVs are not committed; their SHA-256, session kind and the N range searched for each are the table in tests/songs/edit_init/README.md. The searches: fresh sessions N 0..30000 step 16, the first session N 0..60000 step 16. A check of the committed export.sh (Initialize Loop on init_loop.rbs, fresh) matched at N 6016.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each failing export of task-306 is explained with evidence (a fitted orb render equal to the export), or the difference is found and fixed in orb
- [ ] #2 docs/song_load.md section 14's open item is updated: [C] or the fix
<!-- AC:END -->
