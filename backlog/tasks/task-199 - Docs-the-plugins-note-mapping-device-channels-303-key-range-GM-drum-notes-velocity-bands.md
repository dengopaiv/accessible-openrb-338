---
id: TASK-199
title: >-
  Docs: the plugin's note mapping (device channels, 303 key range, GM drum
  notes, velocity bands)
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-16T07:47:00Z'
updated_date: '2026-11-16T09:43:00Z'
labels:
  - docs
  - re
milestone: m-7
dependencies:
  - TASK-185
  - TASK-207
references:
  - backlog task-185 (final summary)
  - docs/midi.md
  - docs/standard_midi_mapping.md
  - docs/initial_investigation.md
  - crates/orb-engine/src/tb303.rs
  - crates/orb-engine/src/tr808.rs
  - crates/orb-engine/src/tr909.rs
priority: medium
type: docs
ordinal: 212000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
task-185 defines orb's own note mapping for the plugin, separate from the original's Standard Mapping (which stays as it is on its own channel, off by default in the plugin: task-207). It is used by note mode (live play), pattern mode's pattern selection and the pattern drag-out. Write it down as one reference, in a new section of docs/midi.md, with what is still to be found:

- **Channels:** 303 #1 = 1, 303 #2 = 2, 808 = 3, 909 = 4 (1-based; changed from 2..5 on 2026-11-16, task-207). The Standard Mapping, when turned on, uses a channel outside 1..4 (default 16).
- **Pattern mode:** notes 0..31 = patterns 1..32 (banks A-D x 8), latched.
- **303 in note mode:** pitch = note; find the MIDI note of ReBirth's C at the Tune knob's center (is ReBirth's tuning concert pitch? the 303's pitch computation, crates/orb-engine/src/tb303.rs and the DLL), and the range: every note the 303's oscillator computes sensibly and audibly (pattern steps reach about 3 octaves with up/down; live notes may go further if the pitch computation allows). Accent = velocity >= 100; overlap = slide; same-pitch overlap = tie; last-note priority, legato.
- **Drums (General MIDI):** BD 36, RS 37, SD 38, CP 39, CH 42, OH 46, LT/MT/HT 43/47/50, 808 congas L/M/H 64/63/62, claves 75, maracas 70, cowbell 56, 808 cymbal and 909 crash 49, 909 ride 51. A shared-slot note plays the voice it names, for that hit only (the switch parameter is untouched).
- **Accent:** 808: velocity < 100 normal, >= 100 AC. 909: step level low x0.8 / high x2.0 times the AC factor 1 + AC*knob/32 (crates/orb-engine/src/tr909.rs): bands 1-47 low, 48-95 high, 96-111 low+AC, 112-127 high+AC. Export velocities for each case (303/808 normal/accent, 909 e.g. 40/80/104/127). Check the 909 gain formula and the 808's against the code and state the loudness order and from which Accent knob value it holds.
- **909 flam:** no note.

Facts about the original found on the way go into their documents with addresses ([C]/[G]).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 docs/midi.md has the note mapping section with every item above
- [ ] #2 The MIDI note of ReBirth's 303 C at Tune center and the 303 range are stated with the engine code or DLL addresses they come from
- [ ] #3 The 808 and 909 accent gains and the band loudness order are stated from the engine code
- [ ] #4 docs/standard_midi_mapping.md points to the new section and says the two mappings use different channels
<!-- AC:END -->
