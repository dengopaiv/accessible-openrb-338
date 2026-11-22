---
id: TASK-284
title: >-
  Keyboard Shortcuts help: the user's review cuts (no menu repeats, no
  mouse-only rows, no TR-/TB- names)
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-22T16:51:00Z'
labels:
  - ui
milestone: m-9
dependencies:
  - TASK-281
references:
  - crates/orb-ui/src/key_help.rs
  - docs/ui_spec.md
priority: medium
type: enhancement
ordinal: 260250
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user reviewed Help > Keyboard Shortcuts (crates/orb-ui/src/key_help.rs) on 2026-11-22, after the 909 editor (task-281) merged, and asked for these changes. The user is the first reviewer of the result (they check it in the app before the agent review).

**Remove:**
1. The note "Keys with the command key held are left to the menu."
2. The row "Cmd+E: Opens the focused row pattern editor (303, 808, 909)". The Edit menu shows the shortcut already.
3. Cmd+E and Cmd+Z from every pattern editor group: menu shortcuts are not repeated there.
4. The note "Both keyboard modes, whatever row has the focus."
5. The note "egui cannot tell the numpad from the main keyboard, so these keys are resolved by the mode and the focus."
6. The row "Numpad *: Nothing: egui gives no key for it (Insert records)".
7. The note "Every key and mouse action of the editor's window; the panel's keys work as without it, except while a value is typed." (it was in both the 808 and 909 groups).
8. The row "Shift / Cmd / Option / Ctrl + click or drag: The same as without: the editor reads no modifier".
9. Every DragValue row: "Drag up / down: On the pattern number or Length: change it" and any other row describing dragging, typing into or Shift+dragging a DragValue (the pattern number, Length, Shuffle and the like). egui's built-in DragValue behavior (Shift+drag is 1/10 speed) is not listed.
10. The "808 Pattern Editor" and "909 Pattern Editor" groups entirely.
11. The "The panel's TR-909 step buttons" group.

**Rename:**
- The group title "Always active" becomes "Global".
- Drop the TR- and TB- prefixes everywhere in the help: "TB-303 #1" becomes "303 #1", "TR-808" becomes "808", "TR-909" becomes "909", and so on in titles, rows and notes. The user's rule: no direct references to the original Roland hardware. This task covers the help's text. Other UI text keeps its names unless the user widens the rule; list any TR-/TB- text found elsewhere in the UI in the notes for the user.

**Keep:** every other row, including the 303 Pattern Editor group's own keys (minus Cmd+E and Cmd+Z).

**Tests and docs:** update key_help.rs's tests to the new groups. Add tests that would fail if a removed row, note or group came back or a TR-/TB- name reappeared. Update docs/ui_spec.md (the Help > Keyboard Shortcuts section and wherever sections 24 and 25 or the panel's 909 step buttons point at the removed groups).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each of the 11 removals is gone from Help > Keyboard Shortcuts
- [ ] #2 "Always active" reads "Global" and no TR- or TB- prefix appears in the help's text
- [ ] #3 key_help tests pass and guard the removals and the renames; docs/ui_spec.md matches
- [ ] #4 TR-/TB- text elsewhere in the UI is listed in the notes, unchanged
- [ ] #5 cargo test --workspace, clippy -D warnings and nightly fmt --check pass
- [ ] #6 The user has checked the help in the app
<!-- AC:END -->
