---
id: TASK-182
title: 'Decide: the key map of screen reader mode'
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-05T19:31:00Z'
updated_date: '2026-11-05T19:31:00Z'
labels:
  - decision
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/keys.rs
  - crates/orb-ui/src/shortcuts.rs
  - docs/accessibility.md
  - docs/accessibility_plan.md
priority: high
type: spike
ordinal: 205000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-D6; the analysis is docs/accessibility.md. Read both before starting.

- Why: four keys collide with NVDA or with focus navigation (Tab, Up/Down, Insert, the
  keypad with Num Lock off), and the new functions need keys.
- Proposal to decide on:
  - F6 / Shift+F6: next / previous section. Tab / Shift+Tab: next / previous control.
  - Up, Down, Page Up, Page Down, Home, End: change the focused value. Left / Right: move in the
    step editor and the grids. Enter: press or toggle (with D2's answer).
  - The 303 tap keys move from Tab / Shift+Tab to backslash / Shift+backslash.
  - Record also gets Ctrl+R (Insert keeps working when it is not the NVDA key).
  - The row focus keys (Up/Down of the original) are replaced by the section moves.
  - F2: read the focused section's pattern. F3: read the transport. F4: read the levels.
  - Ctrl+Up / Ctrl+Down: the original's row focus, for users who want it without moving the
    navigation focus.
- Recommendation: the proposal as it stands; the user changes what does not suit their NVDA
  setup.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user has chosen; the final summary records the choice and the date
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
**Decided (2026-11-05).** The function keys take the transport (with D2), in the layout the
user chose, "transport on F9 to F12": the read-outs low, the transport high and away from F6,
so a slip while moving between sections does not start the music. The 303 tap keys are
Ctrl+Tab and Ctrl+Shift+Tab, close to the original's Tab. The key map of screen reader mode:
- F1: Help > Keyboard Shortcuts.
- F2: read the focused section's pattern. F3: read the transport. F4: read the levels.
- F5: Play / Stop.
- F6 / Shift+F6: next / previous section.
- F7, F8: free.
- F9: Record on / off. (Insert keeps working for users whose NVDA key it is not.)
- F10: the menu bar (and Alt pressed and released alone, A11Y-11).
- F11: Loop on / off.
- F12: song / pattern mode.
- Tab / Shift+Tab: next / previous control.
- Space: press or toggle the focused control (D2).
- Enter: the original's meaning, whatever has the focus (D2).
- Up, Down, Page Up, Page Down, Home, End: change the focused value. Left / Right: move in
  the step editor and the grids.
- Ctrl+Up / Ctrl+Down: the original's row focus, without moving the navigation focus.
- Ctrl+Tab / Ctrl+Shift+Tab: tap a note / a rest into the playing step of the focused 303
  (the original's Tab / Shift+Tab). Note that `App::keyboard` today skips every key pressed
  with Ctrl (`app.rs:725`, the menu shortcuts), so this key needs its own handling; egui does
  not move its focus on Ctrl+Tab (`memory/mod.rs:595`, Tab only without modifiers).
- Every other key of the original (the pitch, drum and pattern keys, minus, equals, the
  digits, the keypad) keeps its meaning, whatever has the focus.
<!-- SECTION:FINAL_SUMMARY:END -->
