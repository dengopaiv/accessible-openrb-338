---
id: TASK-216
title: >-
  Decide: pattern mode in the plugin: who starts and stops it, and a stable
  editing state
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-16T12:56:00Z'
updated_date: '2026-11-16T13:33:00Z'
labels:
  - decision
  - player
  - ui
milestone: m-7
dependencies:
  - TASK-185
references:
  - docs/plugin.md
  - docs/midi.md
  - docs/standard_midi_mapping.md
  - backlog task-185 (final summary)
  - crates/orb-plugin/src/params.rs
priority: high
type: spike
ordinal: 223000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why (the user's Reaper check, 2026-11-16):** pattern mode as built (task-185, task-198) needs rethinking.
1. Once orb is in Play (pattern mode) the pattern keeps playing, also while the host is stopped (orb free-runs). The only ways to stop or mute it are host parameters: the mode `g1` (Play/Stop) or a section's On/off. Automation or the host can overwrite either of them. The user expects a stable pattern-editing state that host MIDI and parameter input do not disturb.
2. There is no way to stop a pattern from MIDI. Notes 0..31 on channels 1..4 select patterns (latched). The Standard Mapping's transport notes are dropped (task-185: "the mode is a host parameter and the transport the host's").

**What is built now (docs/plugin.md sections 3 and 6):**
- `g1` is a host parameter: Play = pattern mode, Stop = note mode (notes play the devices).
- The pattern mode's step follows the host position while the host plays; orb free-runs while the host is stopped.
- Section notes select patterns, and notes never change parameters.

**Questions and options:**
- **Q1. What starts and stops pattern playback.**
  - (a) Unchanged: `g1` plus free-run.
  - (b) The host transport: in pattern mode orb sounds only while the host plays, and is silent when it stops (no free-run). The panel's Play while the host is stopped is a local audition, not a parameter and not saved.
  - (c) Both, chosen by an Options setting ("Follow host transport").
- **Q2. Is the mode (Play/Stop) a host parameter.**
  - (a) Yes, as now: automatable and learnable, and it can be overwritten.
  - (b) No: plugin state only, changed from the panel and from MIDI (Q3), never by automation.
- **Q3. Stopping from MIDI.**
  - (a) A per-section stop note on channels 1..4. For example note 32, just above the pattern selects: it silences that section's pattern until the next pattern-select note, like the original's pattern 'off'.
  - (b) The Standard Mapping's transport notes on the remote channel (default 16; off by default) start and stop orb, instead of being dropped.
  - (c) Both.
  - (d) None: the host's MIDI learn on the mode parameter only. This needs Q2 (a).
- **Q4. A stable editing state.**
  - (a) Nothing more beyond Q1 to Q3.
  - (b) An Options toggle "Lock panel": while it is on, pattern-select notes and stop notes are ignored and the panel's pattern selection stays where the user put it. Host automation of the knobs still applies, because the host owns parameter values.
  - (c) Decouple editing from playback: the pattern being edited is chosen on the panel and is never changed by MIDI. MIDI selects only what plays. This is a larger UI change, with a separate 'playing' indicator.

**Recommendation:**
- Q1 (b): host transport by default, with the panel's Play as audition while the host is stopped. This is what users of other step-sequencer instruments expect: nothing sounds while the DAW is stopped unless you audition it.
- Q2 (b): the mode leaves the host parameters, so automation can't fight it. `g1` is removed; the parameter id list changes once, before any release.
- Q3 (c): stop note 32 on channels 1..4 (documented in docs/midi.md section 5), plus the Standard Mapping's Play/Stop when it is on.
- Q4 (b), the lock toggle, as a cheap guard. (c) only if the user wants an editor separate from playback.

**After the decision:** a feature task (m-7, @agent-complex) implements it in crates/orb-plugin and docs/plugin.md and docs/midi.md, with tests in crates/orb-plugin/tests. task-185's final summary gets a change note.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user has chosen an option for Q1 to Q4 (or written their own), recorded in the final summary with the date
- [x] #2 A follow-up implementation task exists with the chosen behaviour, labels, milestone m-7 and dependencies
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user on 2026-11-16, with their own design in place of options Q1 to Q4.

- **orb Play/Stop enter and leave a pattern preview.** Preview is the standalone app's pattern mode inside the plugin. While preview runs, all host input is ignored: notes, MIDI and parameter changes. The panel plays its own patterns at the song's tempo, free-running, whatever the host's transport does.
- **Preview edits are a separate preview set.** Knobs, switches and pattern selections changed during preview live in a preview set kept in the plugin's state. Leaving preview returns playback and the panel to the host's parameter values; the host never sees the preview edits. The preview set comes back the next time preview is entered. The first entry starts from the current values. Pattern step edits are pattern data, so they persist as before.
- **A separate pattern/note mode setting in the plugin's state,** not a host parameter, so automation can't flip it. It is a panel or Options control, saved with the project.
  - Pattern mode plays only while the host plays, with the step following the host position as now. When the host is stopped there is no sound, and pattern-select notes only change the panel's selected patterns.
  - Note mode: notes play the devices, as now.
- **The mode leaves the host parameters,** so `g1` is removed as a host parameter. Stopping from MIDI is the host's transport stop in pattern mode; no stop note is added.
- The Lock panel toggle (Q4) is not needed, because preview is the stable editing state.

The implementation is task-217.
<!-- SECTION:FINAL_SUMMARY:END -->
