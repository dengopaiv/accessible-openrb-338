---
id: TASK-189
title: >-
  MIDI Standard Mapping: the Program Synth from Keyboard notes (step programming
  by MIDI)
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-15T12:31:00Z'
labels:
  - player
  - ui
  - re
milestone: m-7
dependencies:
  - TASK-182
references:
  - docs/standard_midi_mapping.md
  - crates/orb-player/src/midi.rs
  - crates/orb-ui/src/model.rs
priority: low
type: feature
ordinal: 204000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
task-182 implemented the original's MIDI Standard Mapping as a source-independent layer (crates/orb-player/src/midi.rs) but left out one part: in the keyboard mode "Program Synth from Keyboard" (mode 10, set by MIDI notes 95 and 64 or the Options menu), notes 12..63 program steps of the focused section instead of selecting patterns. docs/standard_midi_mapping.md section 7 has the mapping from the binary (sub_4402AB for the 303s: pitch keys, octave, accent, slide, note/pause, back, pitch mode, pressed on note-on and released on note-off; sub_4435D9 / sub_447680 for the 808 / 909: notes 12..27 toggle step 1..16 of the selected instrument, 28..39 select the instrument, the 909's note 40 toggles a byte that is probably the Flam button). In orb the step editor lives in orb-ui's panel model (Panel::press with the 303 key ids, set_step808/909, select_instrument), not in player commands, so the layer needs a way to express these as panel actions (for example an Action that names the panel button and whether it is pressed or released) that the standalone app (task-188) and the plugin can apply. Follow the binary, including OB-25 (notes 30 and 31 both press Back), or record a deliberate difference in docs/standard_midi_mapping.md section 10.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 In Program Synth mode the layer maps notes 12..63 of the focused section as docs/standard_midi_mapping.md section 7 says, tested with byte streams
- [ ] #2 A test in crates/orb-ui shows the step programmed by MIDI on the panel and in the engine's pattern
- [ ] #3 docs/standard_midi_mapping.md section 7 and 10 are updated
<!-- AC:END -->
