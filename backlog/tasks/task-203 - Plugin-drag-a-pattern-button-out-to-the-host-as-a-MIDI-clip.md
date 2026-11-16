---
id: TASK-203
title: 'Plugin: drag a pattern button out to the host as a MIDI clip'
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-16T07:47:00Z'
updated_date: '2026-11-16T18:51:00Z'
labels:
  - ui
  - player
milestone: m-7
dependencies:
  - TASK-185
  - TASK-199
  - TASK-201
  - TASK-223
references:
  - backlog task-185 (final summary)
  - docs/midi.md
  - docs/ui_spec.md
  - docs/song_load.md
priority: medium
type: feature
ordinal: 216000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Dragging a section's pattern button from the plugin editor (task-201) onto the DAW drops that section's pattern as a MIDI clip, as task-185's final summary decides. Drag only: no "Save MIDI file" fallback, no song or all-patterns export, no import. The user reviews the result before merge.

- **The clip** (a Standard MIDI File, type 0 or 1 with one track): in the note mapping of docs/midi.md (task-199): the section's channel (1..4: 303 #1 = 1, 303 #2 = 2, 808 = 3, 909 = 4; task-207); 16th-note steps at the pattern's length; 303 pitch, slide as overlap, tie as one long note, accent as velocity; drums as GM notes, the 808 shared slots by the switch's current position, accent and the 909 levels as the documented export velocities (e.g. 40/80/104/127). Shuffle: decide straight or baked and document it. A test plays the clip back through note mode (task-200, if merged) or compares against the pattern's step data.
- **The drag:** a press-and-move on a pattern button (a click still selects the pattern; a movement threshold tells them apart) starts an OS drag of a temporary .mid file. egui/baseview cannot start an OS drag, so this is native code per platform: macOS (an AppKit dragging source with a file URL), Windows (OLE `DoDragDrop` with CF_HDROP), Linux (XDND). Clean up the temporary files.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The MIDI writer has tests for a 303 pattern (pitch, slide, tie, accent, gate), an 808 pattern (shared slots, AC) and a 909 pattern (four levels, flam)
- [x] #2 Dragging a pattern button into Reaper on macOS creates a MIDI item with the expected notes; Windows and Linux drags are implemented and tested where a machine is available (else stated)
- [x] #3 A click without movement still selects the pattern
- [x] #4 The user reviewed the drag in a host and approved it before merge
- [x] #5 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. orb-plugin src/clip.rs: the clip writer. From one pattern's bytes (303: 2 + 16x6, drums: 2 + 16x12), the 808's five shared-slot switches and the shuffle value, build the notes and a type-0 Standard MIDI File (one track, PPQ 192, so 1 MIDI tick = 1 engine tick; the section's channel 1..4; a track name; 4/4; end of track at the pattern's length, 48 ticks a step).
   - 303: simulate the step machinery's gate (0x100253A0 step, note-on 0x10023CD4, note-off 0x10023C74: the gate opens at the step, closes 24 ticks later unless the step slides). A legato step (after a slide, a rest step too, since a rest still moves the pitch) becomes an overlapping note; the same pitch is merged into one long note (a tie). Key = 36 + note + 12*up - 12*down; velocity 80 / 127 (accent).
   - Drums: the GM notes of docs/midi.md 5.4, the 808 slots by their switches, and the export velocities of 5.6 (808 80/127; 909 40/80/104/127, flam lost); a length of 24 ticks.
   - Shuffle: baked, as the engine's shuffled tick (seq.rs shuffled, 0x1001F3D0) gives the step and gate times when the pattern's Shuffle is on.
   - Tests: a parser of the written file and checks for a 303 (pitch, slide, tie, accent, gate, rest after a slide), an 808 (switches, AC) and a 909 (four levels, flam) pattern, plus shuffle.
2. orb-ui: Embed::drag_pattern(row, pattern) (default: no drag). In the plugin's panel a press on a pattern button that moves DRAG_OUT points or more calls it once per press. The press still selects the pattern at mouse down, as before. The standalone is unchanged.
3. orb-plugin: EditorEmbed::drag_pattern builds the clip from the plugin's patterns and values, writes it to a fresh temporary folder under temp/OpenRB-338/clips, and hands the path to EditorApp. After the frame, EditorApp starts the native drag from the window's raw handle (src/drag.rs).
   - macOS: posted to the main queue; NSView beginDraggingSessionWithItems with an NSURL file URL and a dragging source class registered under a unique name per image (ClassBuilder, as the vendored raw-window-metal); at the end a synthesized mouseUp to baseview's view.
   - Windows: posted with a thread timer; OLE DoDragDrop with hand-made IDataObject (CF_HDROP) and IDropSource; then WM_LBUTTONUP to the window.
   - Linux: XDND on a connection and thread of its own, if it compiles here.
   - Everything runs under editor::guard. The clips are removed by a sweep of the folders older than 10 minutes, at each drag and at editor close.
4. Tests: the plugin's tests/editor.rs: a press-and-move calls the drag once with the right clip; a click without movement selects the pattern and drags nothing. On Windows, a cfg(windows) test of the data object.
5. Docs: docs/plugin.md (a new section on the drag-out: the clip is meant for note mode, the shuffle choice, cleanup, per-platform notes) and docs/midi.md 5.6 (the tie as one long note, the rest after a slide, the shuffle).
6. Checks: cargo test, clippy, nightly fmt, cargo xwin clippy for Windows. Build the bundles with cargo xtask bundle orb-plugin --release and run scripts/validate-plugin.sh. The task stays To Do with a draft summary for the user's Reaper check.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-acb4e589ba2e0e2f2 (worktree .claude/worktrees/agent-acb4e589ba2e0e2f2), based on main 1349508.

Clip writer (crates/orb-plugin/src/clip.rs). The 303 gate model is from tb303.rs step (0x100253A0, 0x100254D3: gate_off = v + 24), note_on (0x10023CD4) and note_off (0x10023C74, skipped while the step slides).
- A rest step keeps the gate and still moves the pitch, so a rest after a slide is legato.
- Shuffle: seq::shuffled (0x1001F3D0). Even steps start on time and odd ones s = shuffle/6 ticks late. Every gate ends s ticks late: even step off at 48k+24+s, odd step on at 48k+s and off at 48k+24+s. The test the_step_and_gate_times_are_the_engines_shuffled_ticks checks every shuffle value 0..127 and every step against seq::shuffled.
- Playback check (clip tests, play_303): two Tb303s at 12 ticks a block, one on the step path, one with the parsed clip's notes through live_note_on/off. They are bit-identical in both sound models, straight and with shuffle 72 and 77 (s = 12), over the gate, the slides, the tie and the rest after a slide.
- Failed hypothesis: that the merged tie and the rests are exact. They are not. (1) A tie written as one note loses the second step's own accent: the first difference is at the tied step, block 12. (2) A rest after a closed gate sets pitch = target in the release tail (note_on without slide), cutting short a glide. For example, after the slide 51 -> 29 the pitch was 54234 against a target of 41984 six blocks later. Note mode's tail glides on, and the phase differs from then on. In the 5-step case the tail at the rest has a peak of 0.558 and the largest difference is 0.020. Both are kept as limits of the clip, documented and asserted by the_clip_cannot_carry_a_rest_or_a_ties_own_accent. The tie stays one note as the task asks, since overlapping notes of one pitch are corrected or re-triggered by hosts.

Platform checks (2026-11-16):
- macOS: cargo test 824 passed, 0 failed, 5 ignored (88 test binaries); cargo clippy --all-targets -D warnings clean.
- Windows: CL=-Wno-incompatible-pointer-types XWIN_ACCEPT_LICENSE=1 cargo xwin clippy --target x86_64-pc-windows-msvc -p orb-plugin --all-targets -- -D warnings is clean. The first try had 2 errors: an unused ReleaseStgMedium, which is now cfg(test), and a constant assertion, now a const block.
- Windows: cargo xwin test --no-run built the unit-test exe, which was run on the Windows 11 test machine. It took 1 permit of redc-win-run, the exe went to rb/task203/ and was removed afterwards; script scratchpad/worktree-agent-acb4e589ba2e0e2f2/wintest.sh. With the filter drag:: clip::, 12 passed, including drag::windows::tests::the_data_object_holds_the_file_as_a_drop: QueryGetData, EnumFormatEtc through SHCreateStdEnumFmtEtc, GetData CF_HDROP read back with DragQueryFileW, and the QueryContinueDrag answers. DoDragDrop itself was not run.
- Linux: the workspace's Linux check from the Mac fails in the build scripts of alsa-sys and ogg_next_sys (no x86_64-linux-gnu-gcc), and Docker is not running. So drag/x11.rs was checked in a scratch crate (scratchpad/.../x11check, the real file through #[path], with nice-plug and x11rb): cargo clippy --target x86_64-unknown-linux-gnu --all-targets -D warnings is clean. Linking needs a Linux linker. It was not run against an X server.
- Decide task created: task-223 (whether a drag-out also selects the pattern; the recommendation is to keep it). task-203 depends on it.

Final checks on the rebased branch (main 9371a35):
- cargo +nightly fmt --all --check: ok.
- cargo clippy --all-targets -- -D warnings: ok.
- cargo test: 827 passed, 0 failed, 5 ignored (88 binaries).
- Standalone app: the --screenshot PNGs (the empty panel, and 303_accent0.rbs) are byte-identical between this branch and main's app.rs and embed.rs (sha1 0639eb52 and 67f159af; script scratchpad/.../shots.sh).
- Bundles: cargo xtask bundle orb-plugin --release built target/bundled/OpenRB-338.clap and .vst3; scripts/validate-plugin.sh reports both validators passed (clap-validator, pluginval).
- macOS unit test drag::macos::tests::the_source_class_and_the_file_url: the source class is registered once with both methods, the operation mask is copy, the view is kept and given back, and the NSURL file URL is percent-encoded.
- Not verified: a real drag into a host on any platform. That is AC #2's Reaper part and AC #4, the user's check.

Review round 1 (send back), fixes:
1. A played step after a rest that slid, with the gate closed. In the engine (note_on_semis, the if !self.slide test) such a step gets no envelope restart, and its pitch glides from the rest's pitch, which never sounded. Note mode attacks a note when nothing is held, so the clip cannot carry this. It is now the third documented limit: a comment in clip.rs notes_303, docs/midi.md 5.6 and plugin.md 11. The test the_clip_cannot_carry_a_rest_or_a_ties_own_accent pins it with the pattern [note 0] [rest, slide, pitch 0] [note 0]: the notes are 0-24 and 96-120, and the first difference is at block 8 (step 2), as the reviewer found. The 'two exceptions' and unqualified 'bit for bit' claims are corrected in docs/midi.md, plugin.md, clip/tests.rs and the commit message. docs/midi.md now also explains the slide out of the last step into step 0 of the next loop: the original carries the slide flag over the loop, while the clip, one loop long, ends the note and a looping host attacks step 0.
2. Correction: task-223 is decided. The user kept option 1 on 2026-11-16, so a drag-out also selects the pattern. The 'decision pending' text is removed from docs/plugin.md 11 and from the final summary.
3. Dependencies: my earlier backlog task edit --dep task-223 replaced the list instead of adding to it. TASK-185, TASK-199 and TASK-201 are restored alongside TASK-223.
4. docs/plugin.md line 7 and docs/midi.md line 352 are rewrapped to 100 columns.

Review round 1 checks: cargo +nightly fmt --check ok; cargo clippy --all-targets -D warnings ok; cargo test 827 passed, 0 failed, 5 ignored (the new limit is an assertion in an existing test); the bundles were rebuilt and scripts/validate-plugin.sh passed both validators. The Windows-only and X11 code is unchanged since round 0.

Rebase onto main f84d164 (task-218, Gate Pattern Mode):
- Conflicts in crates/orb-ui/src/embed.rs: DRAG_OUT is kept next to task-218's PluginMode docs.
- Conflicts in docs/plugin.md: both intros are kept. Section 11 is now task-218's Gate Pattern Mode, and the drag-out moved to section 12. The cross-references are updated in clip.rs, drag.rs, task-223's refs and summary, and this task's summary; the earlier notes above say 'plugin.md 11' and mean today's section 12.
- docs/plugin.md 12 now names both pattern modes: Latch selects and Gate plays while held.
- Checks: cargo +nightly fmt --check ok; clippy --all-targets -D warnings ok; cargo test 843 passed, 0 failed, 5 ignored. The bundles were rebuilt and scripts/validate-plugin.sh passed both validators.

Rebase onto main ff22a15 (task-224, the editor resize-loop fix):
- Conflicts: in Cargo.lock, orb-plugin's dependency list keeps task-224's objc2-app-kit and objc2-foundation (its macOS dev-dependencies). In crates/orb-plugin/tests/editor.rs, task-224's new tests and this task's drag tests are both kept.
- Checks: cargo +nightly fmt --check ok; clippy --all-targets -D warnings ok; cargo test 844 passed, 0 failed, 5 ignored. The bundles were rebuilt and scripts/validate-plugin.sh passed both validators.

User check: the user tried the drag-out from e5a92d8's bundles in Reaper on macOS on 2026-11-16 and said "drag-out looks fine". That checks AC #2's host part and AC #4 for macOS. Windows and X11 are built and unit-tested, and Windows' data object also passed on the Windows 11 test machine, but no real drag has run on either platform.
Rebase onto main c91b55f (task-202) had no conflicts. Checks: cargo +nightly fmt --check ok; clippy --all-targets -D warnings ok; cargo test 851 passed, 0 failed, 5 ignored. The bundles were rebuilt and scripts/validate-plugin.sh passed both validators.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Dragging a pattern button out of the plugin editor now drops that pattern into the host as a MIDI clip. The drag starts when a press on one of a section's 8 pattern buttons moves 6 points or more. A click still selects the pattern only. The press selects the pattern at mouse down, so a drag-out also selects it; the user kept this behavior (task-223, 2026-11-16).

The clip (crates/orb-plugin/src/clip.rs):
- A type-0 SMF, PPQ 192, on the section's channel 1..4, with no tempo.
- 303: the notes follow the engine's gate. A slide is a 12-tick overlap; a tie is one long note; a rest after a slide is legato; accent is velocity 127, else 80.
- Drums: GM notes, the 808's shared slots by their switches, and the export velocities (808 80/127; 909 40/80/104/127; the flam is lost).
- Shuffle is baked in, as the engine's shuffled tick.
- The 303 clip played back in note mode is bit-identical to the step path (both sound models, straight and shuffled) for patterns without the three things the clip cannot carry: a tie's own accent, a rest's effect on the release tail, and a played step after a rest that slid. Each one is documented and pinned by a test. A slide out of the last step into the next loop is lost too, since the clip is one loop.
- The clip is meant for note mode (docs/plugin.md 12, docs/midi.md 5.6).

The native drag (src/drag.rs) is posted after the egui frame:
- macOS: an AppKit dragging session with the file's NSURL.
- Windows: OLE DoDragDrop with CF_HDROP.
- Linux: an XDND source on its own X connection.
- Everything runs under editor::guard. Temporary files under temp/OpenRB-338/clips/ are swept after 10 minutes.

Verified:
- The user's Reaper check on macOS, 2026-11-16: "drag-out looks fine".
- On main c91b55f: cargo test 851 passed; clippy and nightly fmt are clean.
- The bundles validate.
- The standalone app's screenshots are byte-identical to main's.
- Windows: cargo xwin clippy is clean, and the data-object and clip tests passed on the Windows 11 test machine.
- Linux: the X11 module passes clippy for Linux in a scratch crate.
- No real drag has run on Windows or X11.

Branch worktree-agent-acb4e589ba2e0e2f2, one commit. Decision: task-223.
<!-- SECTION:FINAL_SUMMARY:END -->
