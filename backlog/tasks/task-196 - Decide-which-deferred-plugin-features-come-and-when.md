---
id: TASK-196
title: 'Decide: which deferred plugin features come, and when'
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-16T07:46:00Z'
updated_date: '2026-11-16T16:02:00Z'
labels:
  - decision
  - ui
  - player
milestone: m-7
dependencies:
  - TASK-185
references:
  - backlog task-185 (final summary, 'Later')
priority: low
type: spike
ordinal: 219000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
task-185 (2026-11-15/16) settled the first version of the VST3/CLAP plugin and deferred these items. For each: do it, when (m-7 or later), and the open design questions. Context for each is in task-185's final summary and notes.

1. **Song mode in the plugin.** Play the loaded .rbs song synced to the host. Open: host tempo vs song tempo; host bar 1 = song bar 1 or an offset; locate/chase on seek and loop jumps (the controller state at the new position); host automation of recordable controls vs the song's tracks (the last write wins, or the song locks them); the host's loop wins over orb's.
2. **Recording as import.** orb's song-mode Record in the plugin, with the result saved as an .rbs. Open: where the recording goes (the plugin state), how it interacts with the host's own automation recording.
3. **Per-device pattern/note override.** Each section (303 #1, 303 #2, 808, 909) chooses pattern mode or note mode on its own, instead of the one Play/Stop switch. Natural with the per-device channels 2..5.
4. **Both voices of an 808 shared slot at once** (tom+conga, rim+claves, clap+maracas) in note mode. Needs a second voice per shared slot, used only in note mode (pattern playback keeps one voice, bit-exact), and a UI showing both.
5. **303 visual feedback in note mode.** What lights when a 303 plays a note (e.g. a keyboard key, the step LEDs).
6. **Plugin MIDI output.** The patterns played out as MIDI, so the host records them or drives other instruments (nice-plug supports note output).
7. **Multi-out (stems).** Per-device or per-voice aux outputs, ReWire's channel set (manual pp. 105-107: Mix L/R, 303-1, 303-2, 808-Mix, 909-Mix, per-voice groups; delay return on the mix only). Needs engine taps; stereo vs mono; the master compressor.
8. **Accent threshold setting.** Note mode uses a fixed velocity 100 (303, 808) and fixed 909 bands; a setting in the plugin.
9. **.rbs export from the plugin.** File > Save As for the instance's patterns (Save is hidden now; edits live in the DAW project).
10. **Zoom presets** in the plugin editor (now only host resizing and the invisible resize corner).

Recommendation: 3 and 8 first (small, follow directly from the note mapping); 6 and 7 next (common plugin features); 1, 2 and 4 when users ask; 5, 9 and 10 as polish.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Each of the ten items has an outcome (do now in m-7, later, or never) with the date in the final summary
- [x] #2 Each item to do has its implementation task created with labels, milestone and dependencies
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user on 2026-11-16: none of the ten deferred items goes into the first plugin release. All ten are deferred until after that release, which ships what is already planned in m-7:
- Gate Pattern Mode (task-218);
- the logo animation only for a new instance (task-219);
- note-mode visual feedback (task-202);
- pattern drag-out as MIDI (task-203);
- the lost-parameter bug (task-214);
- Help > Plugin Usage (task-221);
- the first release run (task-215).

No implementation tasks were created. Revisit this list after the release, with the earlier recommendation as the starting point: 3 and 8 first, then 6 and 7, then 1, 2 and 4 on demand, then 5, 9 and 10 as polish. The remote MIDI channel (task-208, task-209) is deferred the same way.
<!-- SECTION:FINAL_SUMMARY:END -->
