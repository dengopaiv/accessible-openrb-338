---
id: TASK-221
title: 'Plugin: Help > Plugin Usage, the in-app guide to modes and MIDI mappings'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-16T15:59:00Z'
labels:
  - ui
  - docs
milestone: m-7
dependencies:
  - TASK-218
  - TASK-219
  - TASK-202
  - TASK-203
  - TASK-209
references:
  - docs/plugin.md
  - docs/midi.md
  - docs/standard_midi_mapping.md
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/embed.rs
  - backlog task-185 (final summary)
  - backlog task-216 (final summary)
priority: high
type: docs
ordinal: 228000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why (the user, 2026-11-16):** documentation is the last thing before the first VST3/CLAP plugin release. A plugin user needs one place that explains how to drive the plugin from a DAW. It goes into the plugin editor's Help menu as "Plugin Usage": an in-app page in orb-ui's style, readable at the plugin's window sizes, scrollable, opened from the menu strip. The plugin has no keyboard shortcuts, so it is opened and closed with the mouse.

**Write it last.** This task depends on the remaining plugin features, so the guide describes the released behavior. Take the facts from the code and from docs/plugin.md, docs/midi.md and docs/standard_midi_mapping.md, not from memory; where they disagree, fix the doc that is wrong and say so in the notes.

**Content, short and task-oriented (what to put in the DAW and what happens):**
1. **Modes (Options menu).** Latch Pattern Mode, Gate Pattern Mode and Note Mode: what each does with the host playing and stopped. The preview on the panel's Play/Stop: it ignores host input, keeps its own preview set and uses the host's tempo. Which mode a new instance starts in.
2. **Channels.** Devices on MIDI channels 1..4: 1 and 2 the ORB-303s, 3 the ORB-808, 4 the ORB-909. The Standard Mapping (remote) channel, 16 by default and off by default, and how to turn it on (task-209's UI).
3. **Pattern selection (both pattern modes).**
   - Notes 0..31 select patterns A1..D8 (0 = A1, 7 = A8, 8 = B1, 31 = D8). Show them as note names in the DAW's usual octave naming, and say which naming is used.
   - Latch: when a change takes effect (the next bar line).
   - Gate: start and stop at once, the last held note wins, the host-position step.
   - Notes never change parameters, so automation doesn't record MIDI pattern changes; the clip is the record.
4. **Note mode mappings.**
   - ORB-303: the key range (MIDI 0..83, C at Tune center = MIDI 36, concert pitch). Accent is velocity >= 100. Slide comes from overlapping notes, and an overlap of the same pitch is a tie.
   - ORB-808 and ORB-909: the GM drum notes per instrument, as a table.
   - Velocity: the 808 accents at >= 100. The 909 has four bands: 1-47, 48-95, 96-111, 112-127.
   - The shared-slot voices of the 808: what picks the voice.
   - The section On/off switch and note mode.
   - The visual feedback (task-202).
5. **Host integration.**
   - Every visible control is a host parameter, ORB-303 1 Cutoff and so on. Automate it or MIDI-learn it with the host's tools.
   - What isn't a parameter: the mode, the preview set, pattern data.
   - Latency of 64 samples; the state is saved with the project, including the mod.
   - Dragging a pattern out as MIDI (task-203).
6. **A short "Recipes" part:**
   - a Reaper-style setup with one MIDI track per device, each sent to the plugin track with its channel;
   - arranging with Gate mode;
   - live pattern switching with Latch.

**Also:** the same text as docs/plugin_usage.md, generated from or kept identical to the in-app source by a test, so it can be linked from the README and the release notes. The user reviews the page in the plugin before merge: screenshots at 1x and 2x, plus the user's check in Reaper.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The plugin editor's Help menu has Plugin Usage, opening a scrollable in-app page that works at the plugin's window sizes without keys
- [ ] #2 The page covers the modes and preview, channels, pattern selection in Latch and Gate, note-mode mappings (303 range, accent, slide/tie; GM drum notes; velocity bands), host parameters/automation, latency/state, drag-out, and recipes
- [ ] #3 Every fact is checked against the code and docs/plugin.md/midi.md; disagreements fixed and listed in the notes
- [ ] #4 docs/plugin_usage.md holds the same text, kept identical by a test, and the README links it
- [ ] #5 Screenshots at 1x and 2x; the user reviews the page in Reaper before merge
- [ ] #6 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->
