---
id: TASK-189
title: Announcements and the read-out keys
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-05T19:32:00Z'
updated_date: '2026-11-06T08:22:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-186
  - TASK-182
references:
  - crates/orb-ui/src/app.rs
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/model.rs
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: high
type: feature
ordinal: 212000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-5; the analysis is docs/accessibility.md. Read both before starting.

- What: `crates/orb-ui/src/a11y/announce.rs`: one node, polite live region, outside the focus
  order, whose text is the last message (repeated messages must be spoken again: add a
  changing invisible part or clear and set in two frames; test with NVDA which works). It
  speaks: the status line's messages; a song opened (its name and mod); a mod chosen; the
  keyboard mode changed (Ctrl+F, Ctrl+G); what an original key did when the focus is not on the
  control it changes (pattern selected with the pattern keys, tempo up/down, Record, Loop, mode,
  Play/Stop with F5 or Enter, Record with F9). The status line itself also gets a node with its
  text. Read-out
  keys (D6): F2 reads the focused section's pattern (A11Y-6 and A11Y-7 fill in the synths and
  drums; here the pattern row: "Pattern B3, 16 steps, shuffle off"), F3 the transport ("Playing,
  bar 5, 128 BPM, song mode, loop 1 to 8, recording off"), F4 the levels (master left and right,
  compressor gain reduction; in dB where A11Y-14 finds the engine's scale, else as numbers).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Tests: each listed event sets the announcer's text; nothing sets it during 10 seconds of playback without input (position, meters and LEDs moving).
- [x] #2 Tests: F2, F3 and F4 set the expected texts on a known song.
- [x] #3 In the real window with NVDA running (version recorded in the notes), a message repeated twice raises a UI Automation LiveRegionChanged event each time (narrowed by the user, 2026-11-06; what NVDA speaks is TASK-197's)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. crates/orb-ui/src/a11y/announce.rs: `Announcer` (the last message and a serial), owned by `App` (`App::announcer`); `App::announce(text)` speaks only in screen reader mode. At the end of `App::show` the announcer is one node (role Label, live Polite, no focus, no bounds); each message is a NEW node (its id carries the serial), so that accesskit_windows raises LiveRegionChanged for every message, a repeated one included (adapter.rs:256 raises it for an added node with a name; node_updated, adapter.rs:313-321, only when the name changes).
2. The status line gets its own node (role Status, label the text, bounds the bar) in `App::status_bar`. Every status message goes through one `App::set_status` that also announces it (menu.rs: save, export start/end, refresh mods, preferred variant; error_box.rs `notify`).
3. Song opened (`adopt_as`, after the load: "Opened NAME, mod MOD"), File > New ("New song, mod Standard ReBirth"), a mod chosen (`use_kit_inner`: "Mod NAME"), the keyboard mode (Ctrl+F, Ctrl+G and the Options menu, one `App::set_key_mode`: "Program synth from keyboard" / "Select patterns from keyboard").
4. Original keys: `App::act` snapshots the state an announcement can name (play, record, loop, mode, tempo, bar, the four rows' patterns) before and after the action and announces the changes whose control does not have egui's focus (`announce::KeyState`, `announce::key_effect`); Play/Stop is always said. A held tempo/bar arrow key announces its final value at the release. TASK-188's F5/F9/F11/F12 get their announcement by going through `App::act` (Action::PlayStop, Action::Press("transport.record"/"transport.loop"/"transport.mode")).
5. Read-out keys, screen reader mode only, one call from `App::keyboard` (`App::readout_key`, in announce.rs): F2 the focused section's pattern row (the row of the section egui's focus is in, else the row focus): "Bass Line 1, pattern B3, 16 steps, shuffle off"; F3 the transport ("Playing, bar 5, 128 BPM, song mode, loop 1 to 8, recording off"); F4 the levels: master left and right of 10 and the gain reduction LEDs of 4 (numbers: the meter scale is [G], not read in the original, docs/ui_spec.md section 16; A11Y-14 found no dB scale for the meters).
6. Tests (crates/orb-ui/tests/a11y_announce.rs): each event sets the text; a repeated message makes a new node with the same text; 10 s of engine time of Trakka Rekka playing with no input: position, meters and LEDs move and the announcer stays; F2/F3/F4 texts on Trakka Rekka; unit tests of the texts in announce.rs. Docs: ui_spec.md section 20.3 (20.2 is TASK-188's), accessibility.md 3.5, key_help.rs (F2-F4 in screen reader mode).
7. Release build, a short NVDA check (AC3) as far as it can be observed.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a80b60ad7bcb7eaba (worktree .claude/worktrees/agent-a80b60ad7bcb7eaba).

Research (2026-11-06):
- accesskit_windows 0.35.2 adapter.rs: node_added (line 256) queues UIA_LiveRegionChangedEventId when the node has a name and live != Off; node_updated (lines 313-321) only when live changed, the node was filtered out before, or the name changed. A Label's UIA name is its value (node.rs:404, label_comes_from_value: Role::Label). So the same text set again on the same node raises nothing.
- Choice for repeats: each message is a new node (egui id from the message's serial), so every message, a repeat included, is a node_added with a name: one frame, no invisible characters in the text, no second frame. Also a new UIA element (new runtime id) per message, so a screen reader that suppresses a repeat from the same element cannot drop it.
- `live` is inherited from the parent (accesskit_consumer node.rs:942); the announcer's parent is not live, so nothing else becomes a live region. Hidden nodes are filtered out (filters.rs:22), so the announcer is not hidden: a Label with no bounds and no focus.
- Role::Status maps to UIA StatusBar (accesskit_windows node.rs:194) with no implied live setting: the status line node does not speak by itself (the announcer does).
- Meters: the meter scale is [G] (docs/ui_spec.md section 16, `sub_424220`, `sub_424A7E` not read); link.rs:879-908 maps the engine's bytes to the 11 meter states and the 4 gain-reduction LEDs. No dB scale is known, so F4 speaks the states as numbers.

Implementation (2026-11-06):
- crates/orb-ui/src/a11y/announce.rs: `Announcer` (text + serial; `node_id()` = egui id ("orb announcement", serial)), `show` (Label, value = label = text, Live::Polite, no bounds, no focus; built last in `App::show`), `describe_status` (role Status, label, bounds; id `status_id()`), `KeyState` + `key_effect` (playing, mode, record, loop, tempo, bar, the 4 rows' patterns; a change whose control has egui's focus is left out: tempo/bar spin buttons, record/loop toggles, mode switch, the row's pattern selector; Play/Stop always), `with_arrow_from` (a held arrow key announces only its own quantity at the release: tempo for tempo keys, bar for bar/wind keys, so playback's bar and song-mode patterns are not attributed to it), the read-outs `pattern_readout`, `transport_readout`, `levels_readout`, and `impl App { announce, set_status, announce_key_effect, readout_key }`.
- app.rs: fields `announcer`, `key_focus` (egui focus when the frame's keys are read), `Hold::seen`; `keyboard()` calls `readout_key` once (screen reader mode, no modifiers, no repeat) and `release(k)` for key-ups; `act()` snapshots `KeyState` before and calls `announce_key_effect` after; `set_key_mode` (Ctrl+F/G and the menu); status node in `status_bar`.
- menu.rs/error_box.rs: every `self.status = Some(..)` became `self.set_status(..)` (toggle_preferred, refresh list, saved, exporting, exported, notify); "Opened NAME, mod MOD" at the end of `adopt_as` (after a successful load), "New song, mod MOD" in `new_now`, "Mod NAME" at the end of `use_kit_inner`.

For TASK-188 (key routing): its F5/F9/F11/F12 are announced for free when they go through `App::act` (Action::PlayStop, Action::Press("transport.record" / "transport.loop" / "transport.mode")); otherwise take `announce::KeyState::of(&self.panel)` before the change and call `self.announce_key_effect(&before)` after. Its row-focus announcement can call `self.announce(text)`. The only line in `App::keyboard` that is mine is the `readout_key` call just after the key-up branch: the rebase conflict is there.
Help > Keyboard Shortcuts: not changed here. F2/F3/F4 belong in the screen reader mode key list that TASK-188 adds to key_help.rs (its AC4); two agents editing key_help.rs at once would conflict. The supervisor should make sure the F2-F4 rows are in that list when both are merged.

Tests (release):
- crates/orb-ui/tests/a11y_announce.rs, 9 tests pass: status message ("Saved PATH") sets the text, the node is a Label, Live::Polite, no Focus action, label = value; the status line node (role Status, no live); the same message again is a new node with the same text and the old node is gone, one live node in the tree; song opened ("Opened Trakka Rekka.rbs, mod Standard ReBirth"), File > New ("New song, mod Standard ReBirth"), mod chosen (assets/Standard.rbm: "Mod Standard ReBirth"); Ctrl+G / Ctrl+F (said again when repeated); original keys in Patterns mode: Num2 "Bass Line 1 pattern A2", X "TR-909 pattern A2", + "Tempo 141 BPM", - "Tempo 140 BPM", Insert "Recording on", / "Loop on", Enter "Playing", Num0 "Stopped, recording off"; the 303's key C announces nothing; focus on the tempo spin button: + is silent (NVDA speaks the value), focus on row 1's pattern selector: Num4 silent; a held + key (about 1.1 s) announces "Tempo N BPM" at the release; with the mode off nothing is announced (serial 0, no node) but the status node exists.
- 10 s of engine time (431 engine calls, 303 frames) of Trakka Rekka playing with no input: the announcer's serial, text ("Playing") and node id unchanged in every frame, while the bar went to 6, the 909's running light changed 92 times, the meters were lit in more than half the frames and the gain-reduction LEDs changed 10 times.
- Read-outs on Trakka Rekka (orb info: 139 BPM, song mode, loop off): F2 (no focus, row focus 0) "Bass Line 1, pattern A6, 16 steps, shuffle off, pattern section off" (the song has Bass Line 1's section off); F3 "Stopped, bar 1, 139 BPM, song mode, loop off, recording off"; F4 "Master left 0 of 10, right 0 of 10, gain reduction 1 of 4" (the LEDs hold the load's engine calls); F2 with the focus on the 909's level fader "TR-909, pattern B5, 16 steps, shuffle off"; after 2 s of play F3 "Playing, bar 2, ..." and F4 "Master left 4 of 10, right 2 of 10, gain reduction 1 of 4" (= levels_readout of the panel).
- Unit tests in announce.rs: 6 pass (texts, the focus rule, the held-arrow rule, rows of sections).
- Failed hypothesis on the way: Enter in the default Program Synth mode with the row focus on a 303 is the 303's Step, not Play; the playback tests switch to Select Patterns mode (Ctrl+G) first.
- Full suite `cargo test --release --no-fail-fast -p orb-ui`: 335 passed, 1 failed (crash.rs a_deliberate_panic_exits_101_and_writes_the_log, the known TASK-200 failure), 3 ignored. frame_cost.rs passes. `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo +nightly-2026-10-18 fmt --all --check` clean.

The real window (AC3, 2026-11-06): NVDA 2026.3.1rc1 running (process 5960, nvda.log "Starting NVDA version 2026.3.1rc1 AMD64"), the release build of OpenRB-338 with the default setting (Automatic: the system reports NVDA, so screen reader mode is on). A Python script (comtypes, UIAutomationCore, scratchpad live_events.py) started the app, registered a UIA LiveRegionChanged (20024) handler on its window's subtree, and posted F3 twice to the window (WM_KEYDOWN/UP, no foreground change). Result: two LiveRegionChanged events, at 4.88 s and 6.42 s, each from an element named "Stopped, 140 BPM, pattern mode": the repeated message raises the event again. What NVDA spoke could NOT be observed: NVDA logs at INFO level (no speech in the log) and the speech viewer is not available to this agent, and NVDA's configuration is the user's and was not changed. So the spoken part of AC3 (and the polite announcement not being cut by a focus change after a system dialog) is left for the user's NVDA pass, TASK-197.
Mishap to record: the first two runs of the script picked a hidden untitled window of the process and sent Ctrl+G, Ctrl+G, Ctrl+F and F3 with keybd_event to whatever window had the foreground then (the app's window was not yet foreground); nothing visible followed. The final script posts the keys to the app's window only.

Review round 1 (2026-11-06): send back. Done in this round:
- Rebased onto main 40c1850 (TASK-188). Conflicts resolved by hand: app.rs imports (announce, drive, nav); keyboard() keeps main's focus block and sets `self.key_focus` after it; ui_spec.md keeps TASK-188's 20.2 and this task's section, now 20.3.
- The read-out call moved inside main's `if screen_reader` block, before the window gate, with `!repeat && !modifiers.any()`; F2, F3, F4 added to the gate's pass list, so the read-outs work with a window open (F2 then reads the row focus's pattern). Test the_read_out_keys_work_with_a_window_open (Keyboard Shortcuts open: F3 "Stopped, 140 BPM, pattern mode", F2 "Bass Line 1, pattern A1, ...").
- Help > Keyboard Shortcuts (key_help.rs screen_reader_group): rows F2 "Read the pattern of the focused section (else of the row with the row focus)", F3 "Read the transport", F4 "Read the levels"; the group's note says F1 to F6, F9, F11, F12 pass a window. New `SrWant::Readout` (maps to None, as keys::map_screen_reader does for F2-F4; `SrWant::Nothing` keeps its meaning).
- Space: `App::activate` wraps `mouse_down` in the KeyState before/after pair, so Space on a focused Play or Stop says "Playing" / "Stopped"; a focused toggle or switch speaks its own state and is not announced. F5, F9, F11, F12 and Enter go through `act` and are announced. Test space_on_play_and_stop_and_the_function_keys_are_announced (Space on Play "Playing", on Stop "Stopped", on the Loop toggle nothing; F5 "Playing", F12 "Song mode", F9 "Recording on", F11 with the Loop toggle focused nothing, F5 "Stopped, recording off").
- B1: AC #3 narrowed to the UIA-event check (the user's decision, 2026-11-06) and checked: the release build with NVDA 2026.3.1rc1 running gave two LiveRegionChanged events for F3 pressed twice (round 0 notes). TASK-197 got AC #5 (NVDA speaks the announcements and the read-outs, a repeat included); docs/accessibility_plan.md NVDA checklist item 21.
- B2: docs/accessibility.md 3.5: the task-189 paragraph is its own paragraph, wrapped; the error box as an alert dialog is marked as TASK-193's, not done (no AlertDialog node exists).
- B3: F4 says "no reading" for a master meter absent from `Panel::meters` and for the gain-reduction LEDs when all nine are 0 (the "0" LED is lit by every Link sync, link.rs:905), instead of a 0 presented as a reading.
- B4: `App::notify` (error_box.rs) goes through set_status, but it has no production caller today (only tests call it); its announcement is untested in the real app.
- B5: the announcement texts use names::display_text and names::pattern_text (tempo, bar, pattern length, patterns), not value_text / control_value_text: none of them is a knob or fader, so D5's units do not apply.
- B6: with the mode off the announcer is inert (no text, no node), but `App::act` and `activate` still take a KeyState before and after each key and build the effect string, which `announce` then drops; the status line's Status node exists with the mode off too.
- B7: F4's gain reduction is a count of lit LEDs, said in docs/ui_spec.md 20.3 and in levels_readout's doc comment.
- The section references in announce.rs and a11y_announce.rs doc comments were changed from 20.2 to 20.3 with sed (comment text only; a slip from the Edit-tool rule).

Round 1 results: a11y_announce.rs 11 pass (2 new tests); full release suite `cargo test --release --no-fail-fast -p orb-ui` 366 passed, 1 failed (crash.rs a_deliberate_panic_exits_101_and_writes_the_log, known, TASK-200), 3 ignored; clippy --workspace --all-targets -D warnings clean; nightly-2026-10-18 fmt --check clean. No keystrokes were sent to any window in this round.

Review round 2 (2026-11-06): send back for stale docs. Done: docs/accessibility.md 3.4 says F2 to F4 are done in task-189 (ui_spec 20.3), only Left / Right pending (task-190, task-191); 3.5 describes the real design (a polite live Label node per message, no bounds, no focus); docs/ui_spec.md section 10's window paragraph says F1 to F6, F9, F11 and F12 keep working with a window (F2 to F4 the read-outs, 20.3); the plan's item 6 says section 20.3; the eprintln! diagnostics in tests/a11y_announce.rs are removed (the row focus print became an assertion that it is 0). `cargo test --release -p orb-ui --test a11y_announce`: 11 pass; nightly fmt check clean; clippy clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Announcements and the read-out keys of screen reader mode, on branch worktree-agent-a80b60ad7bcb7eaba: one commit on main 40c1850 (TASK-188), amended after review round 1.

Outcome:
- One announcer (crates/orb-ui/src/a11y/announce.rs): a polite live Label node, no focus, its text the last message. Each message is a new node, so a repeated message raises UIA LiveRegionChanged again (accesskit_windows adapter.rs:256 vs 313). In the real window with NVDA 2026.3.1rc1 running, F3 twice gave two LiveRegionChanged events with "Stopped, 140 BPM, pattern mode".
- Announced in screen reader mode only: every status line message (App::set_status), a song opened, File > New, a mod chosen, the keyboard mode (Ctrl+F/G and the menu), and what a key did when the focus is not on the control that shows it: Play/Stop, mode, Record, Loop, tempo, bar and the rows' patterns. This covers the original's keys and TASK-188's F5, F9, F11, F12 and Enter (through App::act) and Space on a focused button (App::activate). A held arrow key announces its final value on release. Nothing comes from playback. The status line has a Status node.
- F2 (the focused section's pattern row, else the row focus), F3 (the transport), F4 (the master meters and a count of lit gain-reduction LEDs; "no reading" before the engine reports; numbers, because the meter scale is [G]). The read-outs work with a window open too. Help > Keyboard Shortcuts lists them (SrWant::Readout).
- D5: the announcement texts use names::display_text and names::pattern_text, not value_text/control_value_text. None of them is a knob or fader, so the physical units do not apply.
- With the mode off the announcer is inert (no text, no node). KeyState snapshots and effect strings are still built on each key and dropped. The Status node exists with the mode off too.
- App::notify announces through set_status, but it has no production caller today (only tests).

Tests: a11y_announce.rs 11 pass, announce.rs 6 unit tests pass. `cargo test --release --no-fail-fast -p orb-ui`: 366 passed, 1 failed (crash.rs a_deliberate_panic_exits_101_and_writes_the_log, known, TASK-200), 3 ignored. Clippy (workspace, all targets, -D warnings) and nightly-2026-10-18 fmt check are clean.
Left open: what NVDA speaks (AC #3 was narrowed to the UIA events by the user on 2026-11-06). That is TASK-197 AC #5 and item 21 of the NVDA checklist.

Files: crates/orb-ui/src/a11y/announce.rs (new), crates/orb-ui/src/a11y/mod.rs, crates/orb-ui/src/app.rs, crates/orb-ui/src/menu.rs, crates/orb-ui/src/error_box.rs, crates/orb-ui/src/key_help.rs, crates/orb-ui/tests/a11y_announce.rs (new), docs/ui_spec.md (section 20.3), docs/accessibility.md (3.5), docs/accessibility_plan.md (NVDA checklist item 21), backlog task-189, backlog task-197 (AC #5).
<!-- SECTION:FINAL_SUMMARY:END -->
