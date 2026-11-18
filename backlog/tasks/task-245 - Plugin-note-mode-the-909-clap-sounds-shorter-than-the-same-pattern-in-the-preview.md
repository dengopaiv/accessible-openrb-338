---
id: TASK-245
title: >-
  Plugin note mode: the 909 clap sounds shorter than the same pattern in the
  preview
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-18T08:16:00Z'
labels:
  - engine
  - player
milestone: m-7
dependencies: []
references:
  - /Users/pingw33n/Documents/REAPER Media/909_a6_clap.RPP
  - crates/orb-engine/src/tr909.rs
  - crates/orb-plugin/src/core.rs
  - docs/plugin.md
  - task-200
  - task-203
  - task-242
priority: high
type: bug
ordinal: 450
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** The user (2026-11-18) dragged the 909's pattern A6 out of the plugin into REAPER and played it in note mode. The hand clap sounds shorter than in the preview of the same pattern.

**The project.** The user's REAPER project is `/Users/pingw33n/Documents/REAPER Media/909_a6_clap.RPP`: 120 BPM, 44.1 kHz. Read it; do not modify it.

**The notes.** One 2-second MIDI item, "909 A6", is looped, on MIDI channel 4 (status 0x93). It has 192 ticks per quarter note, so a step (a 16th) is 48 ticks, and every note is 24 ticks long, half a step.
- 36 (BD) at velocity 127;
- 42 (CH) and 46 (OH) at velocity 104;
- 39 (clap) at velocity 104, on steps 5 and 13.
- The item ends with CC 123.

**The plugin state.**
- It is in the `<VST ...>` block. Each line is its own base64 chunk: decode the lines one by one and join the bytes.
- Inside is a JSON object with `params`, and `fields.orb`, a JSON string holding the song, the patterns, `mode` (`note`) and `preview`.
- The preview set differs from the parameters only in `d4c1` (the 909's pattern selection: 0 against 5, A6). So the sounds and the mixer are the same in both modes.

**Where to look.**
- docs/plugin.md 9.2 says a 909 note plays its instrument through the step trigger code, and a note-off does nothing. Check that against the clap's behavior:
  - the clap's voice and its envelope or retrigger;
  - the shared RS/CP slot;
  - how the next steps' instruments (CH, OH, BD) affect a sounding clap in pattern playback versus note mode;
  - the velocity bands (104 is value 1 + AC 2) against the pattern step's actual value and accent, as the drag-out (task-203) writes them;
  - the flam;
  - anything the step function does to the clap that `Tr909::key_on` skips.
- Compare a render of pattern A6 in the preview or Latch Pattern Mode with the drag-out's notes in note mode. Use the plugin core with the restored state, as `crates/orb-plugin/tests/note_clock.rs` does. Find where the clap's samples diverge, then find the cause in the code and the original's disassembly, with addresses.

**Fix.** A note must sound as the pattern's step does. A difference that comes from the drag-out's encoding is fixed there. Record the cause in docs/plugin.md 9.2, and in docs/ if it is a fact about the original.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The cause of the shorter clap is identified with evidence (render comparison, code paths, and the original's addresses where it is a fact about the original) and recorded in the task notes and docs/plugin.md
- [ ] #2 A test plays 909 pattern A6 of the user's state (or an equivalent pattern with the clap followed by CH/OH/BD steps) as drag-out notes in note mode and as the pattern, and the clap's samples match (within the documented tolerance of task-242/task-243)
- [ ] #3 Pattern playback and the corpus are unchanged: full cargo test (corpus_golden included) passes; clippy -D warnings and nightly fmt clean
<!-- AC:END -->
