---
id: TASK-218
title: >-
  Plugin Gate Pattern Mode: a section plays its pattern only while its note is
  held
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-16T15:20:00Z'
labels:
  - player
  - ui
  - docs
milestone: m-7
dependencies:
  - TASK-217
references:
  - backlog task-217 (final summary)
  - backlog task-216 (final summary)
  - docs/plugin.md
  - docs/midi.md
  - docs/standard_midi_mapping.md
  - docs/initial_investigation.md
  - crates/orb-plugin/src/core.rs
  - crates/orb-ui/src/embed.rs
priority: high
type: feature
ordinal: 225000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** The user decided this on 2026-11-16. The existing pattern mode (task-198, task-217) is a latch: notes 0..31 on a section's channel select a pattern that keeps playing until the next note. There is no way to stop a section from a MIDI clip, and a mid-song start plays whatever was selected last. A gate mode fixes both, the way Phoscyon's Host Note mode and ABL3's MIDI Trigger mode do: a pattern plays only while its note is held. In a DAW, the clip's note lengths then are the arrangement, and a gap in the clip is silence.

**Modes after this task (the Options menu radio items, plugin state, not a host parameter, as task-217 built):**
- "Latch Pattern Mode": the existing pattern mode, renamed. Its behavior is unchanged.
- "Gate Pattern Mode": new.
- "Note Mode": unchanged.
- Preview (the panel's Play) overrides all three, as task-217 built.
- The default mode for a new instance stays as task-217 set it.

**Gate Pattern Mode (the user's choices of 2026-11-16):**
1. **Note-off stops the section at once.** The pattern stops sounding at the note-off's frame; no bar-line wait. A note-on also starts at once, at its frame (the supervisor's reading of "stop at once" together with 3; the user can object at the Reaper check).
2. **It plays with the host stopped.** A held note plays its pattern at the host's tempo (the song's tempo if the host reports none), starting from step 1. This allows live triggering from a keyboard.
3. **It follows the host while the host plays.** A held note plays from the step at the host's position, like Latch mode, so a note on beat 2 starts mid-pattern in step with the song. It also follows host loops and jumps continuously.
4. **The last note pressed wins.** Several held notes on one channel: the last note-on selects the pattern. Releasing it falls back to the most recent note still held on that channel, or stops the section if none is held. A switch between held notes takes effect at once, at the host-derived step.
5. **Each section gates on its own channel** (1..4) independently.

**Standard Mapping (remote channel):** its pattern notes only select, in both pattern modes (the user chose two modes over a remote latch layer). In Gate mode a remote selection doesn't open the gate. The remote's other actions are unchanged.

**Engine constraint:** the engine, like the original, applies pattern changes and section on/off only at absolute bar lines (docs/initial_investigation.md, around line 268, [C]). Gate's at-once start, stop and switch must not change that path for songs or for Latch mode.
- Build it in the plugin or orb-player layer: for example, gate the section's output and set the pattern and its position directly when the gate opens, or add an explicit immediate path that only the plugin's gate uses.
- Any orb-engine change needs the full corpus run (`ORB_CD_ISO="/Users/pingw33n/Downloads/Rebirth 2.01 (2001 version).iso" cargo test --release -p orb-engine --test corpus_golden`), with zero differences.
- An abrupt stop may click. Note it, and propose a de-click ramp if one is audible, but don't add one without saying so.

**Docs:** docs/plugin.md (the modes) and docs/midi.md section 5 (gate semantics). Add a change note to task-185's final summary (`--append-final-summary`).

**No compatibility work for old plugin states** (the user's rule until the plugin is released).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The Options menu offers Latch Pattern Mode, Gate Pattern Mode and Note Mode; Latch is the former pattern mode unchanged (its tests still pass)
- [ ] #2 Gate: a section sounds only while a note 0..31 on its channel is held; note-on starts and note-off stops at the event's frame; with the host playing the pattern plays from the host-position step and follows loops and jumps; with the host stopped it plays from step 1 at the host's tempo, or the song's if none (tests through the process path)
- [ ] #3 Gate: with several notes held, the last note-on wins; releasing it falls back to the most recent still-held note or stops the section; sections on channels 1..4 gate independently (tests)
- [ ] #4 Remote (Standard Mapping) pattern notes select but do not open the gate in Gate mode (test)
- [ ] #5 No change to song rendering or Latch mode: the engine's bar-line path is untouched, or the full corpus run passes with zero differences if orb-engine changed
- [ ] #6 docs/plugin.md and docs/midi.md describe Gate mode; task-185's final summary has a change note
- [ ] #7 The user checks Gate mode in Reaper before merge (screenshots of the menu included)
- [ ] #8 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->
