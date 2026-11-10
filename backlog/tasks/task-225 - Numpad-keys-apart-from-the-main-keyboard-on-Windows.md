---
id: TASK-225
title: Numpad keys apart from the main keyboard on Windows
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-10T19:48:00Z'
updated_date: '2026-11-10T20:38:00Z'
labels:
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/keys.rs
  - crates/orb-ui/src/key_help.rs
  - docs/ui_spec.md
priority: medium
type: enhancement
ordinal: 243000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user decided on 2026-11-10: "Since ours is a windows fork only, we can have the numpad check." egui 0.37 cannot tell a numpad key from the main-keyboard key it shares a name with (egui-winit: NumpadSubtract = Minus, Numpad0..9 = Num0..9, NumpadEnter = Enter, NumpadDivide = Slash; NumpadMultiply gives no key at all), so docs/ui_spec.md section 10 resolves those keys by mode and focus, and in Program Synth mode (the default) the tempo has no down key: `-` is Note/Pause there. The original (sub_42D1D3, sub_492312, ui_spec 10) keeps the numpad on its own codes: numpad + / - tempo up / down, Enter Play, 0 Stop, 4 / 5 Rewind / Fast forward, 7 / 8 Bar down / up, 1 / 2 locate to the loop start / end, * Record, / Loop, in every mode. The fork builds for Windows only, so it can ask Windows which key it was.

What: on Windows, tell the numpad keys apart and give them the original's meaning in every mode and both screen reader modes: numpad - is tempo down also in Program Synth mode, numpad digits are transport keys also in Select Patterns mode, numpad Enter is Play and numpad / is Loop whatever has the focus, numpad * is Record, numpad . (VK_DECIMAL) does nothing, as in the original (today it may reach the 303's B pitch key). The main-keyboard keys keep exactly what they do today (the key right of 0 stays Note/Pause / AC in Program Synth mode and tempo down in Select Patterns mode, main-row digits as today), so nothing a user has learned moves.

How (to research, record the choice in the notes): winit's Windows message hook (EventLoopBuilderExtWindows::with_msg_hook, reachable through eframe's NativeOptions::event_loop_builder) sees each WM_KEYDOWN / WM_KEYUP with its virtual key (VK_SUBTRACT, VK_ADD, VK_NUMPAD0..9, VK_DECIMAL, VK_DIVIDE, VK_MULTIPLY; numpad Enter is VK_RETURN with the extended-key bit 24 of lParam) before egui translates it; tag the matching egui key event as numpad. Polling GetKeyState when egui's event arrives is the fallback, but a tap whose key-up is read in the same frame may be missed; say which was used and why. Keep keys::map testable: the numpad flag is an input, not a Win32 call inside it. Non-Windows builds keep today's behavior (cfg), as the fork does not build them.

Note for NVDA users: numpad keys reach the program only with Num Lock on; with Num Lock off NVDA's desktop layout takes them. Say so in Help > Keyboard Shortcuts and ui_spec 10.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 On Windows, numpad - lowers the tempo in Program Synth mode (with a 303 or a drum machine focused) and in Select Patterns mode; the main key right of 0 does exactly what it does today
- [x] #2 Numpad digits, Enter, / and * act as the original's transport keys in both keyboard modes; numpad . does nothing; main-row digits, Enter and / are unchanged
- [x] #3 keys::map takes the numpad flag as an input and has unit tests for each numpad key in both modes and for the unchanged main keys; screen reader mode routes the same way
- [x] #4 Help > Keyboard Shortcuts (key_help.rs), docs/ui_spec.md section 10 ("How OpenRB-338 maps the keys", the egui paragraph and "Where OpenRB-338 differs") and docs/accessibility_plan.md describe the new mapping and the Num Lock / NVDA note
- [x] #5 cargo test --release -p orb-ui, clippy and nightly fmt pass
- [x] #6 An NVDA checklist item for the user's pass (TASK-197): with Num Lock on, numpad - and + change the tempo in Program Synth mode and the value is announced as for the tempo buttons
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. keys.rs: map() and map_screen_reader() get a numpad: bool parameter; numpad true = the original's numpad code in every mode/focus (+ - tempo, 0 stop, 1/2 locate, 4/5 wind, 7/8 bar, Enter Play, / Loop; 3 6 9 . nothing). Unit tests per key, both modes, main keys unchanged.
2. New module numpad.rs: Win32 msg hook (cfg windows) pushes (egui Key, pressed) of numpad VKs into a static queue; numpad * (VK_MULTIPLY, no egui key) is queued separately. App::keyboard claims the queue per egui Key event (same key and pressed), takes * first and treats it as Insert (Record, same action and hold), clears the queue at the end of each frame (hook runs before egui gets the event in the same pump, so each frame's events have their entries). Non-Windows: no hook, queue empty.
3. main.rs: NativeOptions::event_loop_builder installs the hook (cfg windows). app.rs: pass the flag; SR-mode numpad Enter is Play regardless of the section.
4. key_help.rs rows + note on Num Lock / NVDA, docs/ui_spec.md s10, docs/accessibility_plan.md (TASK-225 entry + NVDA checklist item for TASK-197).
5. cargo test --release -p orb-ui, debug test, clippy, nightly fmt.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-aeb1fc2cbf8578487
Research: winit 0.30.13 (platform_impl/windows/event_loop.rs:412) calls the msg hook for every PeekMessageW message before TranslateMessage/DispatchMessageW, on the event-loop thread, so it sees WM_KEYDOWN/KEYUP before winit turns them into events and egui reads them. eframe 0.37 NativeOptions::event_loop_builder takes a FnOnce(&mut EventLoopBuilder<UserEvent>); EventLoopBuilderExtWindows::with_msg_hook takes FnMut(*const c_void)->bool (the pointer is a MSG). windows-sys is in Cargo.lock only transitively (0.52/0.59/0.61), not a dependency of orb-ui; no new dependency is needed: a repr(C) prefix of MSG (hwnd, message, wParam, lParam) is read in the hook.

Design and findings.
- Chosen: the winit message hook (not GetKeyState polling). numpad.rs: hook() reads the first four fields of the MSG (own repr(C) prefix, no windows-sys dependency) for WM_KEYDOWN/KEYUP/SYSKEY*, queues (egui key, pressed) for VK_NUMPAD0-9, VK_ADD, VK_SUBTRACT, VK_DIVIDE, VK_DECIMAL, VK_RETURN+extended bit 24, and VK_MULTIPLY (no egui key). App::keyboard claims the oldest entry with the same key and direction for each egui key event, drops older unclaimed entries, and clears the queue at the end of each frame. A tap (key-up read in the same frame as key-down) works, since the queue holds both events in order; polling would miss it, which is why polling was not used. Held tempo arrows: the numpad key-up event is claimed as numpad too, and release(k) ends the hold by key as before.
- Dependency: winit 0.30.13 added to orb-ui as a [target.'cfg(windows)'.dependencies] entry (needed for the EventLoopBuilderExtWindows trait; eframe re-exports only EventLoopBuilder). Cargo.lock: one line (orb-ui lists winit); no new crate. windows-sys is not needed.
- Failed hypothesis: pairing by egui's physical_key for numpad '.'. egui-winit 0.37 has no KeyCode::NumpadDecimal mapping, so the event carries the logical key: Period, or Comma on a decimal-comma layout (Estonian). claim() accepts both for the decimal entry; unit-tested.
- Numpad * : act(Key::Insert, Press transport.record) at the press and release(Key::Insert) at the release, i.e. exactly Insert's path (Record button held while the key is down). Mouse-down semantics gave no difficulty because Insert already uses the same hold. Auto-repeat of * is dropped in the hook (bit 30). A window that has the keys in screen reader mode keeps it (as Insert).
- keys::map(k, shift, numpad, mode, focus) and map_screen_reader(k, shift, ctrl, numpad, mode, focus) stay pure; numpad true = map_numpad (original codes, all modes). SR mode: numpad Enter is Play whatever section has the focus (the task-207 section rule applies to the main Enter only).
- Not verified live: no numpad was pressed on the real window in this run (no way to inject VK_NUMPAD keys with a hardware extended bit here); the pairing is unit-tested on synthetic messages and the hook is wired to the real event loop. It is covered by the new NVDA checklist item 28 for TASK-197's pass. Estonian layout and keys matched by physical US position are unchanged.
- Tests: cargo test --release -p orb-ui 162 unit tests (+ integration suites) pass; new: keys (numpad table in both modes and focus rows, SR routing, main keys unchanged), numpad.rs (hook pairing), key_help (numpad binds through map and SR).

Review round 1 (all fixed, commit amended).
1. Stale entries: numpad::end_frame() now runs in App::update after the keyboard gate (whether keyboard() ran or not), so entries never survive their frame; a numpad * pressed while a dialog, text field or the menu bar has the keys is dropped. Unit test: an unclaimed * entry is gone after end_frame. No app-level test of the gate: the queue is a process-global static and the integration tests in tests/ run in parallel threads, so a test that pushes entries would race with other tests' end_frame calls; that is also why optional item 4 (app-level numpad * Record and numpad Enter with a 303 section focused) was not added.
2. Num Lock wording fixed in key_help.rs, docs/ui_spec.md section 10 and docs/accessibility_plan.md item 28: with it off Windows changes only the digits and decimal point; NVDA's desktop layout uses the numpad keys (+ and - among them) for review commands while Num Lock is off.
3. numpad::claim(key, has_physical, pressed): the decimal entry claims only an event without a physical key (egui-winit has no NumpadDecimal mapping; the main comma has physical Comma). Unit test: a main comma/period (physical present) does not take the entry.
5. m-7 count recounted from backlog task list: 43 of 45 done.

Review round 2 (agent-reviewer, on 474112e): code sound (end_frame at the end of App::show, every frame; the decimal claim needs no physical key). One wording defect: the NVDA claim was unverified. The supervisor checked NVDA 2024.1's installed User Guide and Key Commands (C:\Program Files\NVDA\documentation\en): "The Desktop layout makes heavy use of the numpad (with Num Lock off)", "numpad keys require the Num Lock to be turned off to work properly"; numpadPlus alone is "Say all with review", numpadMinus is a command only with the NVDA key. Wording fixed at merge by the supervisor in key_help.rs, ui_spec 10 and accessibility_plan item 28, and ui_spec 20's rewrapped line. cargo test --release -p orb-ui 472 passed, 3 ignored; nightly fmt clean. The app-level tests the review asked for are follow-up task-227.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Numpad keys are told apart from the main keyboard on Windows through winit's message hook (numpad.rs, installed via NativeOptions::event_loop_builder; winit added as a windows-only dependency, no new crate). keys::map / map_screen_reader take a numpad flag: a numpad key is the original's code in every mode, focus and both screen reader modes (+ - tempo, Enter Play, 0 Stop, 1/2 locate, 4/5 wind, 7/8 bar, / Loop, * Record via Insert's path, 3 6 9 . nothing); main-keyboard keys unchanged. Help > Keyboard Shortcuts has a group 'The numpad' with the Num Lock / NVDA note; docs/ui_spec.md section 10 and docs/accessibility_plan.md (TASK-225 row, ground rule, NVDA checklist item 28 for TASK-197) updated. Evidence: AC1/2/3 unit tests in keys.rs (numpad table over both modes and all four focus rows, SR routing, main keys unchanged), numpad.rs (hook pairing on synthetic messages), key_help.rs (every numpad bind through map and map_screen_reader; every mapped key listed incl. numpad). AC5: cargo test --release -p orb-ui 162 unit tests plus all integration suites pass, debug cargo test -p orb-ui passes, cargo clippy --all-targets -- -D warnings clean, nightly fmt --check clean. Not verified live: no physical numpad press on the real window; covered by checklist item 28. Branch worktree-agent-aeb1fc2cbf8578487, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
