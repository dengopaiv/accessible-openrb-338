---
id: TASK-225
title: Numpad keys apart from the main keyboard on Windows
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-10T19:48:00Z'
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
- [ ] #1 On Windows, numpad - lowers the tempo in Program Synth mode (with a 303 or a drum machine focused) and in Select Patterns mode; the main key right of 0 does exactly what it does today
- [ ] #2 Numpad digits, Enter, / and * act as the original's transport keys in both keyboard modes; numpad . does nothing; main-row digits, Enter and / are unchanged
- [ ] #3 keys::map takes the numpad flag as an input and has unit tests for each numpad key in both modes and for the unchanged main keys; screen reader mode routes the same way
- [ ] #4 Help > Keyboard Shortcuts (key_help.rs), docs/ui_spec.md section 10 ("How OpenRB-338 maps the keys", the egui paragraph and "Where OpenRB-338 differs") and docs/accessibility_plan.md describe the new mapping and the Num Lock / NVDA note
- [ ] #5 cargo test --release -p orb-ui, clippy and nightly fmt pass
- [ ] #6 An NVDA checklist item for the user's pass (TASK-197): with Num Lock on, numpad - and + change the tempo in Program Synth mode and the value is announced as for the tempo buttons
<!-- AC:END -->
