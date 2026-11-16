---
id: TASK-217
title: >-
  Plugin pattern preview, and a pattern/note mode setting that follows the host
  transport
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-16T13:33:00Z'
labels:
  - player
  - ui
  - docs
milestone: m-7
dependencies:
  - TASK-216
  - TASK-201
references:
  - backlog task-216 (final summary)
  - backlog task-185 (final summary)
  - docs/plugin.md
  - docs/midi.md
  - crates/orb-plugin/src/params.rs
  - crates/orb-plugin/src/instance.rs
  - crates/orb-plugin/src/core.rs
  - crates/orb-plugin/src/editor.rs
  - crates/orb-ui/src/embed.rs
  - crates/orb-plugin/tests/param_ids.txt
priority: high
type: feature
ordinal: 224000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** The user decided this on 2026-11-16 (task-216's final summary is the spec). Pattern mode as built, with `g1` as a host parameter and orb free-running while the host is stopped, gave no stable way to edit patterns. Host MIDI and automation could change or stop what the user was editing, and nothing could stop a pattern from MIDI.

**What to build (crates/orb-plugin, orb-ui's plugin presentation in embed.rs):**
1. **Pattern preview on the panel's Play/Stop.**
   - Play enters preview and Stop leaves it. Preview behaves like the standalone app's pattern mode: orb plays its patterns free-running at the song's tempo, whatever the host's transport does.
   - While preview runs, all host input is ignored: notes, the Standard Mapping, and parameter changes or automation. Ignoring means the preview's sound and panel don't change; the host's parameter values still update in the plugin's model, so leaving preview returns to the latest host values.
2. **The preview set.**
   - Knob and switch changes and pattern selections made during preview go into a preview set in the plugin's state, not to the host parameters. Send no begin/set/end gestures to the host for them.
   - Leaving preview returns the panel and the engine to the host's parameter values. The next entry restores the preview set. The first entry starts from the current values.
   - Pattern step edits (pattern data) keep working as now and persist whether in preview or not.
   - The preview set is saved and restored with the state; old states without it still load.
3. **A pattern/note mode setting in the plugin's state,** not a host parameter, saved with the project. It is a panel or Options-menu control; pick the clearer one and show a screenshot.
   - Pattern mode plays only while the host plays, with the step following the host position as now (task-198, task-206 rule).
   - When the host is stopped there is no sound, and pattern-select notes (0..31 on channels 1..4) only change the panel's selected patterns.
   - Note mode: notes play the devices, as now (task-200, task-210).
   - Preview overrides both modes while it runs.
4. **Remove `g1` (the mode) from the host parameters.**
   - Update crates/orb-plugin/tests/param_ids.txt and the parameter count. Shuffle `g20` and the others keep their ids. No release has shipped yet.
   - A state saved with `g1` loads and maps it to the new mode setting: Play is pattern mode, Stop is note mode.
5. **Displays.** Sync and Bar show the host while orb follows it and are dark in preview, where the panel's own position shows if the standalone has one in this layout. The MIDI LED still blinks on input during preview, but the input is ignored.
6. **Docs.** Update docs/plugin.md (parameters section 3, the modes and host sync, the state) and docs/midi.md section 5 (pattern selection when the host is stopped; preview ignores input). Add a change note to task-185's final summary (`--append-final-summary`) pointing to task-216.

**Context:** the plugin's mode and transport live in crates/orb-plugin (params.rs, instance.rs, core.rs; docs/plugin.md). The editor's Play/Stop goes through PluginPort in editor.rs. Mind task-214, an open bug: a parameter change before the first engine block after an activation is lost; it touches the same activation path. Engine code is unchanged, so no corpus run is needed unless engine code changes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Panel Play enters preview: the patterns play free-running at the song tempo whether the host plays or not, and host notes, Standard Mapping messages and parameter changes during preview change neither the sound nor the panel (tests through the plugin's process path)
- [ ] #2 Changes made in preview are kept in a preview set: leaving preview returns the panel and engine to the host's current parameter values, the next preview restores the set, no host parameter gesture is sent for them, and the set survives a state save/restore (tests)
- [ ] #3 A pattern/note mode setting in the plugin state, not a host parameter: in pattern mode orb sounds only while the host plays (step from the host position); with the host stopped there is no sound and pattern-select notes only change the panel's selected patterns; note mode is unchanged (tests)
- [ ] #4 g1 is no longer a host parameter (param_ids.txt updated); a state saved with g1 loads and maps it to the mode setting (test)
- [ ] #5 docs/plugin.md and docs/midi.md describe preview, the preview set and the mode setting; task-185's final summary has a change note
- [ ] #6 Screenshots of the panel in preview and with the mode control; the user checks it in Reaper before merge
- [ ] #7 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->
