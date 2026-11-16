---
id: TASK-200
title: 'Plugin: note mode, host notes play the 303s, 808 and 909'
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-16T07:47:00Z'
updated_date: '2026-11-16T09:43:00Z'
labels:
  - engine
  - player
milestone: m-7
dependencies:
  - TASK-185
  - TASK-197
  - TASK-199
  - TASK-207
references:
  - backlog task-185 (final summary)
  - docs/midi.md
  - crates/orb-engine/src/tb303.rs
  - crates/orb-engine/src/tr808.rs
  - crates/orb-engine/src/tr909.rs
  - docs/initial_investigation.md
priority: medium
type: feature
ordinal: 213000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
With orb stopped, the plugin is in note mode (task-185's final summary): host notes on the device channels 1..4 (303 #1 = 1, 303 #2 = 2, 808 = 3, 909 = 4; task-207) play the devices, by the note mapping in docs/midi.md (task-199). The original never plays a device from a note (task-183), so this is new engine behavior; pattern playback must stay bit-exact.

- **303:** a monophonic voice driven by notes: pitch from the note, accent at velocity >= 100, slide when notes overlap (the next note slides from the held one), tie on same-pitch overlap, last-note priority, legato, gate = note length. Use the engine's own step machinery (note-on, slide, accent, gate) rather than a new synth, so the sound is the 303's.
- **808 and 909:** each note triggers its voice, as a pattern step would, with the accent factor of its velocity band (808 normal/AC; 909 low/high/low+AC/high+AC). A shared-slot note (tom/conga, rim/claves, clap/maracas) plays the voice it names for that hit only; the switch parameter is not changed. One voice per slot as the engine has it (the second note cuts the first). Chokes and the BD's two alternating voices as in pattern playback.
- Works whether or not the host plays. Notes never change host parameters.
- **Per-voice sounding mask:** the engine exposes, per block, which 808/909 voices are sounding (active and gain above about -60 dB) and whether the voice's last hit had AC, for the note-mode visual feedback task.

Notes in pattern mode keep their pattern-select meaning (task-198).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Tests: a 303 note sequence gives the expected pitch, accent, slide, tie and gate (checked against the engine's step path with an equivalent pattern where possible)
- [ ] #2 Tests: each GM drum note triggers the right voice; shared-slot notes pick their voice without changing the switch parameter; velocity bands give the documented gains
- [ ] #3 The reference corpus is unchanged (corpus_golden passes): pattern playback is bit-exact
- [ ] #4 The per-voice sounding mask and the AC flags are in the snapshot, with a test
- [ ] #5 No allocation in the audio path (no_alloc test)
- [ ] #6 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->
