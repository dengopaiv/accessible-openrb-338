---
id: TASK-226
title: >-
  Plugin: a note at the very start of the host's timeline plays wrong (drums) or
  is skipped in a render (all devices)
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-16T17:32:00Z'
labels:
  - player
milestone: m-7
dependencies: []
references:
  - crates/orb-plugin/src/core.rs
  - crates/orb-plugin/src/lib.rs
  - docs/plugin.md
  - docs/midi.md
priority: high
type: bug
ordinal: 231000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Report (the user, 2026-11-16, Reaper 7.86 on macOS, plugin built from main ff22a15 plus task-202):**
- A MIDI note placed exactly at the song start (position 0) of a Reaper project plays differently from the same note later in the clip when it is a drum: the 808/909 sound is different, for example a wrong velocity/accent, a cut, or a different sample start.
- When the project is rendered to a file, the first note at position 0 is skipped for every device, the 303s included.
- The user thinks this is unrelated to the features merged today.

Assumption to confirm first: this is Note Mode, where host notes play the devices directly. Check also Latch and Gate Pattern Mode with a note at 0 (task-218's gate path).

**Likely areas:**
- The first process block after activation or a transport start: the order in which the core applies the transport change (Locate/Play/Stop, the sync path, preview exit), the parameter resend and the note events at frame 0 of the same block.
- Something there resets the voices after the note at frame 0 has triggered them, or drops events that arrive before the first "playing" block.
- In an offline render Reaper starts processing at sample 0 with the transport just started, possibly after a fresh activate/reset, so a note at frame 0 lands exactly in that first block.
- For the drums, a "different" sound could mean the accent/velocity state, or the per-instrument parameter resend, isn't applied yet when the note fires.
- Check task-214 (the lost parameter change after activation) and task-225 (dropped commands when the queue is full): they may share the cause.

**To do:**
1. Reproduce it in a test: drive the plugin core or the CLAP/VST3 wrapper the way a host renders. Activate, then process from sample 0 with the transport starting and a note-on at frame 0. Compare the output with the same note at a later frame, shifted, for each device: 303 1/2, 808, 909. Cover a realtime start as well as an offline render (CLAP's `process` with steady time 0, VST3's `processContext` with the playing flag set in the first block).
2. Find the cause, with the code path.
3. Fix it, so that a note at frame 0 of the first block sounds the same as the note later, bit for bit after the shift where the model allows.
4. Run the corpus if orb-engine changes. No allocation or lock in process.
5. Rebuild and validate the bundles. The user checks it in Reaper: a note at 0 played live and in a render, for drums and the 303.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The cause is identified with the code path and a failing test before the fix
- [ ] #2 A note-on at frame 0 of the first processed block (realtime start and offline render, CLAP and VST3 paths) produces the same output as the same note later (shifted), for 303 1/2, 808 and 909
- [ ] #3 Latch and Gate Pattern Mode with a note at position 0 are checked too
- [ ] #4 cargo test, clippy -D warnings, nightly fmt pass; corpus 175/175 if the engine changed; validators pass
- [ ] #5 The user checks it in Reaper (live and rendered)
<!-- AC:END -->
