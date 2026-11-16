---
id: TASK-198
title: >-
  Plugin: host sync and pattern mode (notes select patterns, step from the host
  position)
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-16T07:47:00Z'
labels:
  - player
milestone: m-7
dependencies:
  - TASK-185
  - TASK-197
references:
  - backlog task-185 (final summary)
  - docs/ui_spec.md
  - docs/song_load.md
  - docs/initial_investigation.md
priority: medium
type: feature
ordinal: 211000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The plugin (task-197) gets its pattern mode and host sync, as task-185's final summary decides (read it first).

- **Mode switch.** orb's Play/Stop is the mode, an automatable host parameter: playing = pattern mode, stopped = note mode (note mode is its own task). Song mode does not exist in the plugin; File > Open always puts the instance in pattern mode.
- **Pattern mode plays whether or not the host plays.**
  - Tempo: the host's (hosts report it while stopped); the loaded song's when the host reports none.
  - While the host plays, the step comes from the host position: bar 1 beat 1 = step 1. Host start, seek and loop jumps realign it, and an offline bounce is deterministic. Shuffle stays orb's (a parameter).
  - While the host is stopped, orb free-runs from where it is.
- **Pattern selection from notes.** Notes on the device channels 2..5 (303 #1, 303 #2, 808, 909): note n in 0..31 selects pattern n+1 of that section (banks A-D x 8). Latched: it changes at once, at the step the host position gives, and stays until the next note on that channel; note-off is ignored. A section with no note yet plays the plugin state's (the panel's) pattern; a panel click and a host note both change it, the last write wins.
- The Standard Mapping (task-182, docs/standard_midi_mapping.md) keeps working on its own channel (default 1), as in the standalone app; a remote channel equal to a device channel is refused or warned.
- The snapshot gives the editor what it needs: whether orb follows the host position (for the Sync LED: lit in pattern mode while orb and the host both play) and incoming MIDI (MIDI LED).

Where the original's sequencer has no "jump to step" (it advances in ticks per 64-frame block), find the way to place it at a host position (docs/initial_investigation.md, the sequencer and the transport's locate), and record the method in docs.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Tests: with a simulated host transport, the step follows the host position after start, seek and loop jump; a host tempo change changes orb's tempo; with the host stopped orb keeps playing
- [ ] #2 Tests: notes 0..31 on channels 2..5 select patterns at once and latch; note-off ignored; a panel change after a note wins until the next note
- [ ] #3 Two offline renders of the same host timeline are identical
- [ ] #4 The Standard Mapping still works on its channel; a device channel as remote channel is refused or warned
- [ ] #5 The snapshot exposes the Sync LED state (pattern mode, orb and host playing) and a MIDI activity flag
- [ ] #6 How orb places the sequencer at a host position is in docs with the engine addresses it relies on
- [ ] #7 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->
